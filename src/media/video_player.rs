//! 视频播放器（基于 ffmpeg-next 的实现）
//!
//! 设计：
//! - `play(path)` 打开文件，启动一个**单独的解码 std::thread**
//!   （避免 ffmpeg 阻塞调用阻塞 tokio runtime）。
//!   线程内部完成：解封装 → 视频解码 → swscale 到 RGBA8 → 推到 latest_frame；
//!   音频解码 → swresample → rodio Sink 播放。
//! - 同步策略：以**视频 PTS 时钟**为基准。线程持有自播放起始的 `Instant`，
//!   每解出一帧后 sleep 至该帧 `pts` 对应的目标显示时刻再写入 `latest_frame`。
//!   音频独立喂入 rodio sink，由 sink 自带定时驱动播放节奏；
//!   视频追音频实现的"严格 A/V 同步"在此场景下被简化（GalGame 视频
//!   通常较短，5~30 秒级片头/CG），可接受少许漂移。
//! - 取消：`cancel: AtomicBool` 在每次循环开头检查；UI 端 `stop()` 会置位
//!   并立即停止 audio sink，解码线程下一次循环退出。
//! - 完成：解封装 EOF + 解码缓冲 flush 完毕后置 `finished = true`。
//!
//! 资源生命周期：`VideoPlayer` 被 drop 时自动 `stop()`，确保 ffmpeg 资源释放。

use crate::error::MediaError;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type as MediaType;
use ffmpeg_next::software::resampling::context::Context as ResamplingContext;
use ffmpeg_next::software::scaling::{context::Context as ScalingContext, flag::Flags};
use ffmpeg_next::util::format::sample::{Sample as SampleFormat, Type as SampleType};
use ffmpeg_next::util::frame::{audio::Audio as AudioFrame, video::Video as VideoFrame};
use ffmpeg_next::util::rational::Rational;
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once};
use std::thread;
use std::time::{Duration, Instant};

/// 解码线程产出的帧载体。`SharedPixelBuffer` 是 `Send`，可跨线程传递；
/// `slint::Image` 不是 `Send`，因此在 UI 线程消费时再封装。
type FrameBuffer = SharedPixelBuffer<Rgba8Pixel>;

/// ffmpeg 全局初始化（线程安全，仅初始化一次）。
static FFMPEG_INIT: Once = Once::new();

fn ensure_ffmpeg_initialized() {
    FFMPEG_INIT.call_once(|| {
        // 即便 init 失败也只能记录日志——后续 open 会再次报错。
        if let Err(e) = ffmpeg::init() {
            eprintln!("ffmpeg init failed: {e}");
        }
        // 关闭 ffmpeg 的冗余日志（仅保留 ERROR 级别）。
        ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Error);
    });
}

/// 一段视频的播放句柄。
pub struct VideoPlayer {
    path: String,
    cancel: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    latest_frame: Arc<Mutex<Option<FrameBuffer>>>,
    /// 持有解码线程句柄，drop 时 join，确保 ffmpeg 资源被回收。
    decode_thread: Option<thread::JoinHandle<()>>,
}

impl VideoPlayer {
    /// 启动视频播放：打开文件、启动解码线程、立即返回。
    pub fn play(path: &str) -> Result<Self, MediaError> {
        ensure_ffmpeg_initialized();

        // 路径检查给出更友好的错误（ffmpeg 自己的 NotFound 也能识别，但信息更模糊）。
        if !std::path::Path::new(path).exists() {
            return Err(MediaError::OpenFile {
                path: path.to_string(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            });
        }

        let cancel = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let latest_frame: Arc<Mutex<Option<FrameBuffer>>> = Arc::new(Mutex::new(None));

        let path_owned = path.to_string();
        let cancel_thread = cancel.clone();
        let finished_thread = finished.clone();
        let latest_frame_thread = latest_frame.clone();

        let decode_thread = thread::Builder::new()
            .name("video-decoder".to_string())
            .spawn(move || {
                if let Err(e) = decode_loop(
                    &path_owned,
                    cancel_thread,
                    finished_thread.clone(),
                    latest_frame_thread,
                ) {
                    eprintln!("video decode failed: {e}");
                }
                finished_thread.store(true, Ordering::Release);
            })
            .map_err(|e| MediaError::OpenFile {
                path: path.to_string(),
                source: e,
            })?;

        Ok(Self {
            path: path.to_string(),
            cancel,
            finished,
            latest_frame,
            decode_thread: Some(decode_thread),
        })
    }

    /// 请求停止播放。幂等。
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Release);
    }

    /// 视频是否已结束（自然结束或被取消）。
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    /// 取出最新解码出的视频帧（如果有）。
    /// take 语义：UI timer 在每次轮询时取走最新帧后写入 slint，避免重复 set_video_frame。
    pub fn take_latest_frame(&self) -> Option<Image> {
        let buf = self.latest_frame.lock().ok()?.take()?;
        Some(Image::from_rgba8(buf))
    }

    /// 当前播放的文件路径。
    #[allow(dead_code)]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop();
        if let Some(handle) = self.decode_thread.take() {
            // 等待解码线程结束，确保 ffmpeg 资源完全释放。
            // 单元短视频场景下退出延迟通常 < 一帧周期。
            let _ = handle.join();
        }
    }
}

/// 解码循环。所有 ffmpeg 资源都在本函数栈上，函数返回即被释放。
fn decode_loop(
    path: &str,
    cancel: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    latest_frame: Arc<Mutex<Option<FrameBuffer>>>,
) -> Result<(), MediaError> {
    let mut ictx = input(path).map_err(|e| MediaError::DecodeVideo {
        path: path.to_string(),
        reason: format!("open input: {e}"),
    })?;

    // ---- 视频流 ----
    let video_stream = ictx
        .streams()
        .best(MediaType::Video)
        .ok_or_else(|| MediaError::DecodeVideo {
            path: path.to_string(),
            reason: "no video stream".into(),
        })?;
    let video_stream_index = video_stream.index();
    let video_time_base: Rational = video_stream.time_base();

    let video_ctx = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
        .map_err(|e| MediaError::DecodeVideo {
            path: path.to_string(),
            reason: format!("video codec ctx: {e}"),
        })?;
    let mut video_decoder = video_ctx.decoder().video().map_err(|e| MediaError::DecodeVideo {
        path: path.to_string(),
        reason: format!("video decoder: {e}"),
    })?;
    let src_w = video_decoder.width();
    let src_h = video_decoder.height();
    let mut scaler = ScalingContext::get(
        video_decoder.format(),
        src_w,
        src_h,
        Pixel::RGBA,
        src_w,
        src_h,
        Flags::BILINEAR,
    )
    .map_err(|e| MediaError::DecodeVideo {
        path: path.to_string(),
        reason: format!("sws context: {e}"),
    })?;

    // ---- 音频流（可选）----
    let audio_setup = setup_audio(&ictx).map_err(|mut e| {
        // 补齐路径上下文（setup_audio 内部不知道 path）
        if let MediaError::DecodeVideo { path: ref mut p, .. } = e {
            if p.is_empty() {
                *p = path.to_string();
            }
        }
        e
    })?;

    let playback_start = Instant::now();

    // ---- 主循环：读包并按流索引分发 ----
    for (stream, packet) in ictx.packets() {
        if cancel.load(Ordering::Acquire) {
            break;
        }

        if stream.index() == video_stream_index {
            video_decoder.send_packet(&packet).ok();
            drain_video_frames(
                &mut video_decoder,
                &mut scaler,
                &latest_frame,
                playback_start,
                video_time_base,
                &cancel,
            );
        } else if let Some(ref a) = audio_setup {
            if stream.index() == a.stream_index {
                let mut audio_decoder = a.decoder.lock().unwrap();
                audio_decoder.send_packet(&packet).ok();
                drain_audio_frames(
                    &mut audio_decoder,
                    &mut a.resampler.lock().unwrap(),
                    &a.sink,
                    a.target_rate,
                    a.target_channels,
                );
            }
        }
    }

    // ---- flush ----
    if !cancel.load(Ordering::Acquire) {
        video_decoder.send_eof().ok();
        drain_video_frames(
            &mut video_decoder,
            &mut scaler,
            &latest_frame,
            playback_start,
            video_time_base,
            &cancel,
        );
        if let Some(ref a) = audio_setup {
            let mut audio_decoder = a.decoder.lock().unwrap();
            audio_decoder.send_eof().ok();
            drain_audio_frames(
                &mut audio_decoder,
                &mut a.resampler.lock().unwrap(),
                &a.sink,
                a.target_rate,
                a.target_channels,
            );
            // 等待 sink 把缓冲区放完
            while !a.sink.empty() && !cancel.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    finished.store(true, Ordering::Release);
    Ok(())
}

/// 持续从 video decoder 取帧，按 PTS 节奏写到 latest_frame。
fn drain_video_frames(
    decoder: &mut ffmpeg::decoder::Video,
    scaler: &mut ScalingContext,
    latest_frame: &Arc<Mutex<Option<FrameBuffer>>>,
    playback_start: Instant,
    time_base: Rational,
    cancel: &Arc<AtomicBool>,
) {
    let mut decoded = VideoFrame::empty();
    let mut rgba = VideoFrame::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        if cancel.load(Ordering::Acquire) {
            return;
        }
        if scaler.run(&decoded, &mut rgba).is_err() {
            continue;
        }

        // 计算这一帧应当显示的目标时刻。
        if let Some(pts) = decoded.pts() {
            let pts_secs = pts as f64 * f64::from(time_base.numerator())
                / f64::from(time_base.denominator());
            let target = playback_start + Duration::from_secs_f64(pts_secs.max(0.0));
            let now = Instant::now();
            if target > now {
                let wait = target - now;
                // 大睡眠时分多次检查 cancel
                let chunk = Duration::from_millis(20);
                let mut remaining = wait;
                while remaining > Duration::ZERO {
                    if cancel.load(Ordering::Acquire) {
                        return;
                    }
                    let s = remaining.min(chunk);
                    thread::sleep(s);
                    remaining = remaining.saturating_sub(s);
                }
            }
        }

        let buf = video_frame_to_pixel_buffer(&rgba);
        if let Ok(mut slot) = latest_frame.lock() {
            *slot = Some(buf);
        }
    }
}

/// 把 RGBA8 的 ffmpeg 帧转换成 SharedPixelBuffer<Rgba8Pixel>（Send，可跨线程）。
fn video_frame_to_pixel_buffer(rgba_frame: &VideoFrame) -> FrameBuffer {
    let w = rgba_frame.width();
    let h = rgba_frame.height();
    let stride = rgba_frame.stride(0);
    let src = rgba_frame.data(0);

    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    let dst = buffer.make_mut_bytes();
    let row_bytes = (w * 4) as usize;

    if stride == row_bytes {
        // stride 和 row 完美对齐时一次性 copy。
        dst.copy_from_slice(&src[..row_bytes * h as usize]);
    } else {
        for y in 0..h as usize {
            let s = &src[y * stride..y * stride + row_bytes];
            let d = &mut dst[y * row_bytes..(y + 1) * row_bytes];
            d.copy_from_slice(s);
        }
    }

    buffer
}

/// 音频上下文：解码器、重采样、rodio sink 等共生体。
struct AudioSetup {
    stream_index: usize,
    decoder: Mutex<ffmpeg::decoder::Audio>,
    resampler: Mutex<ResamplingContext>,
    target_rate: u32,
    target_channels: u16,
    sink: Sink,
    /// 持有 OutputStream 防止 sink 提前失效。
    _stream: OutputStream,
}

fn setup_audio(
    ictx: &ffmpeg::format::context::Input,
) -> Result<Option<AudioSetup>, MediaError> {
    let audio_stream = match ictx.streams().best(MediaType::Audio) {
        Some(s) => s,
        None => return Ok(None),
    };
    let stream_index = audio_stream.index();

    let audio_ctx = ffmpeg::codec::context::Context::from_parameters(audio_stream.parameters())
        .map_err(|e| MediaError::DecodeVideo {
            path: String::new(),
            reason: format!("audio codec ctx: {e}"),
        })?;
    let decoder = audio_ctx.decoder().audio().map_err(|e| MediaError::DecodeVideo {
        path: String::new(),
        reason: format!("audio decoder: {e}"),
    })?;

    // 目标格式：i16 packed，与 ffmpeg 解码出的 native 格式做 swresample。
    let target_format = SampleFormat::I16(SampleType::Packed);
    let target_rate = decoder.rate();
    let target_channel_layout = decoder.channel_layout();
    let target_channels = target_channel_layout.channels() as u16;

    let resampler = ResamplingContext::get(
        decoder.format(),
        decoder.channel_layout(),
        decoder.rate(),
        target_format,
        target_channel_layout,
        target_rate,
    )
    .map_err(|e| MediaError::DecodeVideo {
        path: String::new(),
        reason: format!("resampler: {e}"),
    })?;

    // 启动 rodio
    let (stream, handle) = OutputStream::try_default().map_err(MediaError::from)?;
    let sink = Sink::try_new(&handle).map_err(MediaError::from)?;

    Ok(Some(AudioSetup {
        stream_index,
        decoder: Mutex::new(decoder),
        resampler: Mutex::new(resampler),
        target_rate,
        target_channels,
        sink,
        _stream: stream,
    }))
}

fn drain_audio_frames(
    decoder: &mut ffmpeg::decoder::Audio,
    resampler: &mut ResamplingContext,
    sink: &Sink,
    target_rate: u32,
    target_channels: u16,
) {
    let mut decoded = AudioFrame::empty();
    let mut resampled = AudioFrame::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        if resampler.run(&decoded, &mut resampled).is_err() {
            continue;
        }
        // packed i16: 所有 channel 交错在 plane 0
        let bytes = resampled.data(0);
        let sample_count = bytes.len() / 2;
        let mut samples = Vec::with_capacity(sample_count);
        for chunk in bytes.chunks_exact(2) {
            samples.push(i16::from_ne_bytes([chunk[0], chunk[1]]));
        }
        let buf = SamplesBuffer::new(target_channels, target_rate, samples);
        sink.append(buf);
    }
}
