use audio_buffer::{
    buffers::{fixed_frames::FixedFrameBuffer, interleaved::InterleavedBuffer},
    core::{Buffer, BufferMut},
};
use audio_engine::{
    memory::BufferStorage,
    timeline::{Clip, Timeline},
};
use interavl::IntervalTree;
use std::{collections::VecDeque, path::PathBuf, time::Duration};
use time::{FrameTime, MusicalTime, SampleRate};

fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut storage = BufferStorage::<InterleavedBuffer<f32>>::new();

    let key = storage.insert(
        audio_buffer::loader::load(assets_dir.join("synth_keys_48000_16bit.wav"))
            .expect("failed to load audio"),
    );

    let mut timeline = Timeline::new(120.0, SampleRate::default(), IntervalTree::default());
    timeline
        .insert(
            MusicalTime::from_fractional_beats::<8>(3, 1)
                ..MusicalTime::from_fractional_beats::<8>(4, 0),
            Clip {
                buffer: key,
                offset: FrameTime::new(0),
            },
        )
        .unwrap();

    // let mut graph = Graph::new();
    // let audio_node_index = graph.add_node(AudioNode {
    //     events: VecDeque::new(),
    // });
    // graph.set_master(Some(audio_node_index));

    // for events in timeline.iter_blocks(FrameTime::new(256)) {
    // let audio_node = graph.node_mut(audio_node_index).unwrap();
    // events
    //     .iter()
    //     .for_each(|event| audio_node.push_event(event.clone()));

    // let mut out = [[0.0; 2]; 256];
    // graph.audio_requested(&mut out, timeline.sample_rate().as_f64());
    // dbg!(out);
    // }

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
