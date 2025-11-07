use std::path::PathBuf;

use audio_engine::{engine::AudioEngine, playlist::Clip};
use time::{FrameTime, MusicalTime, SampleRate};

fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut engine = AudioEngine::<f32>::new(120.0, SampleRate::default(), FrameTime::new(256));

    let buffer = engine
        .load_audio_file(assets_dir.join("synth_keys_44100.ogg"))
        .expect("failed to load audio file");

    let track_idx = engine.add_track();
    let track = engine.get_track_mut(track_idx).expect("is valid");
    let playlist = track.get_playlist_mut();

    playlist.insert(
        MusicalTime::ZERO..MusicalTime::from_quarter_beats(2, 3),
        Clip { buffer },
    );

    engine.run();
}
