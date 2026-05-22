use crate::error::AudioError;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

pub struct Player {
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}

#[derive(Debug, Clone)]
pub enum PreBgm {
    Play(String),
    Stop,
    None,
}

impl Player {
    pub fn new() -> Result<Self, AudioError> {
        let (_stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            sink: Arc::new(Mutex::new(None)),
            _stream,
            stream_handle: handle,
        })
    }

    pub fn play_loop(&self, path: &str, volume: f32) -> Result<(), AudioError> {
        if let Some(s) = self.sink.lock().unwrap().take() {
            s.stop();
        }

        let file = File::open(path).map_err(|e| AudioError::OpenFile {
            path: path.to_string(),
            source: e,
        })?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| AudioError::Decode {
                path: path.to_string(),
                source: e,
            })?
            .repeat_infinite();

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        sink.set_volume(volume);
        sink.play();

        *self.sink.lock().unwrap() = Some(sink);
        Ok(())
    }

    pub fn stop(&self) {
        if let Some(s) = self.sink.lock().unwrap().take() {
            s.stop();
        }
    }

    pub fn change_volume(&self, volume: f32) {
        let mut sink = self.sink.lock().unwrap();
        if let Some(sink) = sink.as_mut() {
            sink.set_volume(volume);
        }
    }

    pub fn play_voice(&self, path: &str, volume: f32) -> Result<(), AudioError> {
        if let Some(s) = self.sink.lock().unwrap().take() {
            s.stop();
        }
        let file = File::open(path).map_err(|e| AudioError::OpenFile {
            path: path.to_string(),
            source: e,
        })?;
        let source = Decoder::new(BufReader::new(file)).map_err(|e| AudioError::Decode {
            path: path.to_string(),
            source: e,
        })?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        sink.set_volume(volume);
        sink.play();

        *self.sink.lock().unwrap() = Some(sink);
        Ok(())
    }
}
