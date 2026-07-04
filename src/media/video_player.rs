use crate::error::MediaError;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type as MediaType;
use ffmpeg_next::software::{
    resampling::context::Context as ResamplingContext,
    scaling::{context::Context as ScalingContext, flag::Flags},
};
use ffmpeg_next::util::{
    format::sample::{Sample as SampleFormat, Type as SampleType},
    frame::{audio::Audio as AudioFrame, video::Video as VideoFrame},
    rational::Rational,
};
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Once,
    },
    thread,
    time::{Duration, Instant},
};

type FrameBuffer = SharedPixelBuffer<Rgba8Pixel>;

static FFMPEG_INIT: Once = Once::new();

fn ensure_ffmpeg_initialized() {
    FFMPEG_INIT.call_once(|| {
        if let Err(e) = ffmpeg::init() {
            eprintln!("ffmpeg init failed: {e}");
        }
        ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Error);
    });
}

pub struct VideoContext {
    video_player: Option<VideoPlayer>,
    video_timer: Option<slint::Timer>,
}

impl VideoContext {
    pub fn default() -> Self {
        VideoContext {
            video_player: None,
            video_timer: None,
        }
    }

    pub fn set_video_player(&mut self, player: VideoPlayer) {
        self.video_player = Some(player);
    }

    pub fn set_video_timer(&mut self, timer: slint::Timer) {
        self.video_timer = Some(timer);
    }

    pub fn get_video_player_ref(&self) -> Option<&VideoPlayer> {
        self.video_player.as_ref()
    }

    pub fn get_video_player(&mut self) -> Option<VideoPlayer> {
        self.video_player.take()
    }

    pub fn get_video_timer(&mut self) -> Option<slint::Timer> {
        self.video_timer.take()
    }
}

/// 一段视频的播放句柄。
pub struct VideoPlayer {
    path: String,
    cancel: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    latest_frame: Arc<Mutex<Option<FrameBuffer>>>,
    decode_thread: Option<thread::JoinHandle<()>>,
}

impl VideoPlayer {
    pub fn play(path: &str) -> Result<Self, MediaError> {
        ensure_ffmpeg_initialized();

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

    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Release);
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    pub fn take_latest_frame(&self) -> Option<Image> {
        let buf = self.latest_frame.lock().ok()?.take()?;
        Some(Image::from_rgba8(buf))
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop();
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
    }
}

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

    // 视频流
    let video_stream =
        ictx.streams()
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
    let mut video_decoder = video_ctx
        .decoder()
        .video()
        .map_err(|e| MediaError::DecodeVideo {
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

    // 音频流
    let audio_setup = setup_audio(&ictx).map_err(|mut e| {
        if let MediaError::DecodeVideo {
            path: ref mut p, ..
        } = e
        {
            if p.is_empty() {
                *p = path.to_string();
            }
        }
        e
    })?;

    let playback_start = Instant::now();

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
            while !a.sink.empty() && !cancel.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    finished.store(true, Ordering::Release);
    Ok(())
}

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

        if let Some(pts) = decoded.pts() {
            let pts_secs =
                pts as f64 * f64::from(time_base.numerator()) / f64::from(time_base.denominator());
            let target = playback_start + Duration::from_secs_f64(pts_secs.max(0.0));
            let now = Instant::now();
            if target > now {
                let wait = target - now;
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

struct AudioSetup {
    stream_index: usize,
    decoder: Mutex<ffmpeg::decoder::Audio>,
    resampler: Mutex<ResamplingContext>,
    target_rate: u32,
    target_channels: u16,
    sink: Sink,
    _stream: OutputStream,
}

fn setup_audio(ictx: &ffmpeg::format::context::Input) -> Result<Option<AudioSetup>, MediaError> {
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
    let decoder = audio_ctx
        .decoder()
        .audio()
        .map_err(|e| MediaError::DecodeVideo {
            path: String::new(),
            reason: format!("audio decoder: {e}"),
        })?;

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
