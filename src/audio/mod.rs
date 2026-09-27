pub mod drone;
pub mod state;

use std::sync::Arc;

use rodio::{OutputStream, OutputStreamHandle, Sink};

pub use state::AudioState;

pub struct AudioHandle {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    _sink: Sink,
    pub state: Arc<AudioState>,
}

impl AudioHandle {
    pub fn start() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let sink = Sink::try_new(&handle).ok()?;
        let state = Arc::new(AudioState::default());
        let source = drone::DroneSource::new(state.clone(), 44100);
        sink.append(source);
        sink.play();
        Some(Self {
            _stream: stream,
            _handle: handle,
            _sink: sink,
            state,
        })
    }
}
