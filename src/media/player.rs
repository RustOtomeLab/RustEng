use crate::error::MediaError;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{cell::RefCell, fs::File, io::BufReader};

pub(crate) struct MediaPlayer {
    bgm_player: Player,
    voice_player: Player,
}

impl MediaPlayer {
    pub(crate) fn new() -> Result<Self, MediaError> {
        let bgm_player = Player::new()?;
        let voice_player = Player::new()?;
        Ok(Self {
            bgm_player,
            voice_player,
        })
    }

    pub(crate) fn change_bgm_volume(&self, volume: f32) {
        self.bgm_player.change_volume(volume);
    }

    pub(crate) fn change_voice_volume(&self, volume: f32) {
        self.voice_player.change_volume(volume);
    }

    pub(crate) fn play_bgm(&self, path: &str, volume: f32) -> Result<(), MediaError> {
        self.bgm_player.play_loop(path, volume)
    }

    pub(crate) fn play_voice(&self, path: &str, volume: f32) -> Result<(), MediaError> {
        self.voice_player.play_voice(path, volume)
    }

    pub(crate) fn stop_bgm(&self) {
        self.bgm_player.stop();
    }

    pub(crate) fn stop_all(&self) {
        self.bgm_player.stop();
        self.voice_player.stop();
    }
}

pub(crate) struct Player {
    sink: RefCell<Option<Sink>>,
    _stream: OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}

#[derive(Debug, Clone, Default)]
pub(crate) enum PreBgm {
    Play(String),
    Stop,
    #[default]
    None,
}

impl Player {
    pub(crate) fn new() -> Result<Self, MediaError> {
        let (_stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            sink: RefCell::new(None),
            _stream,
            stream_handle: handle,
        })
    }

    pub(crate) fn play_loop(&self, path: &str, volume: f32) -> Result<(), MediaError> {
        if let Some(s) = self.sink.borrow_mut().take() {
            s.stop();
        }

        let file = File::open(path).map_err(|e| MediaError::OpenFile {
            path: path.to_string(),
            source: e,
        })?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| MediaError::DecodeAudio {
                path: path.to_string(),
                source: e,
            })?
            .repeat_infinite();

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        sink.set_volume(volume);
        sink.play();

        *self.sink.borrow_mut() = Some(sink);
        Ok(())
    }

    pub(crate) fn stop(&self) {
        if let Some(s) = self.sink.borrow_mut().take() {
            s.stop();
        }
    }

    pub(crate) fn change_volume(&self, volume: f32) {
        let mut sink = self.sink.borrow_mut();
        if let Some(sink) = sink.as_mut() {
            sink.set_volume(volume);
        }
    }

    pub(crate) fn play_voice(&self, path: &str, volume: f32) -> Result<(), MediaError> {
        if let Some(s) = self.sink.borrow_mut().take() {
            s.stop();
        }
        let file = File::open(path).map_err(|e| MediaError::OpenFile {
            path: path.to_string(),
            source: e,
        })?;
        let source = Decoder::new(BufReader::new(file)).map_err(|e| MediaError::DecodeAudio {
            path: path.to_string(),
            source: e,
        })?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        sink.set_volume(volume);
        sink.play();

        *self.sink.borrow_mut() = Some(sink);
        Ok(())
    }
}
