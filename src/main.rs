use std::{collections::BTreeMap, path::PathBuf};

use audio_engine::{
    audio::{cache::AudioBufferCache, load},
    timeline::Timeline,
};
use dsp::Node;
use time::SampleRate;

struct DspNode {}
impl Node<[f32; 2]> for DspNode {
    fn audio_requested(&mut self, buffer: &mut [[f32; 2]], sample_hz: f64) {
        dbg!(sample_hz);
        for sample in buffer {
            // *sample = [sample[0] * 3.0, sample[1] * 3.0];
        }
        // dsp::slice::map_in_place(buffer, |f| {
        //     dsp::Frame::map(f, |s| dsp::Sample::mul_amp(s, 3.))
        // });
    }
}

fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let mut cache = AudioBufferCache::<f32>::new();
    let key = cache
        .insert(load(assets_dir.join("synth_keys_48000_16bit.wav")).expect("failed to load audio"));

    let mut timeline = Timeline::new(120.0, SampleRate::default(), BTreeMap::new());

    // let mut graph = Graph::new();
    // let events = BTreeMap::new();
    // let mut timeline = Timeline::new(120., time::SampleRate(44_000.), events);
    // // timeline.insert(
    // //     MusicalTime::from_fractional_beats::<8>(2, 3),
    // //     Event { payload: todo!() },
    // // );

    // for (i, block_events) in timeline.iter_blocks(FrameTime::new(256)).enumerate() {
    //     println!("{i}");
    //     if !block_events.is_empty() {
    //         dbg!(block_events);
    //         return;
    //     }
    // }

    // let synth = graph.add_node(DspNode {});
    // graph.set_master(Some(synth));

    // let mut buffer = [[3.; 2]; 1];
    // dbg!(buffer);
    // graph.audio_requested(&mut buffer, 44100.);
    // dbg!(buffer);
}
