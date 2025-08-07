use std::thread::spawn;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

pub struct BgmPlayer {
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}

impl BgmPlayer {
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
}


pub async fn play_voice(path: &str) {
    let path = path.to_string();
    tokio::spawn(async move {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();

        let file = File::open(path).unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();

        sink.append(source);
        sink.sleep_until_end();
    });
}