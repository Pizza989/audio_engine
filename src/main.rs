use audio_buffer::{buffers::interleaved_dynamic::InterleavedDynamicBuffer, core::Buffer};
use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use audio_engine::{
    audio::{
        cache::{AudioBufferCache, BufferKey},
        shared_buffer::SharedBuffer,
    },
    timeline::{BlockEvent, Clip, Timeline},
};
// use dsp::{Graph, Node};
use interavl::IntervalTree;
use once_cell::sync::Lazy;
use time::{FrameTime, MusicalTime, SampleRate};

static AUDIO_CACHE: Lazy<ArcSwap<AudioBufferCache<InterleavedDynamicBuffer<f32>>>> =
    Lazy::new(|| ArcSwap::from_pointee(AudioBufferCache::new()));

struct AudioNode {
    events: VecDeque<BlockEvent>,
}

impl AudioNode {
    pub fn push_event(&mut self, event: BlockEvent) {
        self.events.push_back(event);
    }
}

// impl Node<[f32; 2]> for AudioNode {
//     fn audio_requested(&mut self, buffer: &mut [[f32; 2]], sample_hz: f64) {}
// }

fn insert_buffer(buffer: InterleavedDynamicBuffer<f32>) -> BufferKey {
    let current_cache = AUDIO_CACHE.load_full();

    let (new_cache, key) = AudioBufferCache::from_slotmap(current_cache.buffers.clone())
        .insert(SharedBuffer::new(buffer));

    AUDIO_CACHE.store(Arc::new(new_cache));
    key
}

fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let key = insert_buffer(
        audio_buffer::loader::load(assets_dir.join("synth_keys_48000_16bit.wav"))
            .expect("failed to load audio"),
    );

    let buffer = audio_buffer::loader::load::<f32>(assets_dir.join("synth_keys_48000_16bit.wav"))
        .expect("failed to load audio");

    for frame in buffer.iter_frames() {
        println!("{:?}", frame);
    }

    // let mut timeline = Timeline::new(120.0, SampleRate::default(), IntervalTree::default());
    // timeline
    //     .insert(
    //         MusicalTime::from_fractional_beats::<8>(3, 1)
    //             ..MusicalTime::from_fractional_beats::<8>(4, 0),
    //         Clip {
    //             buffer: key,
    //             offset: FrameTime::new(0),
    //         },
    //     )
    //     .unwrap();

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
