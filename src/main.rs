use std::path::PathBuf;

use audio_engine::engine::AudioEngine;
use time::{FrameTime, SampleRate};

fn main() {
    let _assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let _engine = AudioEngine::<f32>::new(120.0, SampleRate::default(), FrameTime::new(256));
    std::thread::park();
}
