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

impl Player {
    pub fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().expect("Failed to open audio output");
        Self {
            sink: Arc::new(Mutex::new(None)),
            _stream,
            stream_handle: handle,
        }
    }

    pub fn play_loop(&self, path: &str) {
        if let Some(s) = self.sink.lock().unwrap().take() {
            s.stop();
        }

        let file = File::open(path).expect("Failed to open BGM file");
        let source = Decoder::new(BufReader::new(file))
            .expect("Failed to decode BGM file")
            .repeat_infinite();

        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(source);
        sink.play();

        *self.sink.lock().unwrap() = Some(sink);
    }

    pub fn stop(&self) {
        if let Some(s) = self.sink.lock().unwrap().take() {
            s.stop();
        }
    }

    pub fn play_voice(&self, path: &str) {
        if let Some(s) = self.sink.lock().unwrap().take() {
            s.stop();
        }
        let file = File::open(path).expect("Failed to open Voice file");
        let source = Decoder::new(BufReader::new(file)).expect("Failed to decode Voice file");

        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(source);
        sink.play();

        *self.sink.lock().unwrap() = Some(sink);
    }
}
