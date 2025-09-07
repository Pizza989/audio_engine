use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use arc_swap::ArcSwap;
use audio::buf::Dynamic;
use audio_engine::{
    audio::{
        cache::{AudioBufferCache, BufferKey},
        load,
    },
    timeline::{BlockEvent, Clip, Timeline},
};
use dsp::Node;
use interavl::IntervalTree;
use once_cell::sync::Lazy;
use time::{FrameTime, MusicalTime, SampleRate};

static AUDIO_CACHE: Lazy<ArcSwap<AudioBufferCache<f32>>> =
    Lazy::new(|| ArcSwap::from_pointee(AudioBufferCache::new()));

struct AudioNode {
    events: VecDeque<BlockEvent>,
}

impl Node<[f32; 2]> for AudioNode {
    fn audio_requested(&mut self, buffer: &mut [[f32; 2]], sample_hz: f64) {}
}

fn insert_buffer(buffer: Dynamic<f32>) -> BufferKey {
    let current_cache = AUDIO_CACHE.load_full();

    let (new_cache, key) =
        AudioBufferCache::from_slotmap(current_cache.buffers.clone()).insert(Arc::new(buffer));

    AUDIO_CACHE.store(Arc::new(new_cache));
    key
}

fn main() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let key = insert_buffer(
        load(assets_dir.join("synth_keys_48000_16bit.wav")).expect("failed to load audio"),
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

    let mut counter = 0;
    let mut past = false;
    for events in timeline.iter_blocks(FrameTime::new(256)) {
        if !events.is_empty() {
            dbg!(events);
            counter += 1;
            past = true;
        } else if past {
            break;
        }
    }
    dbg!(counter);

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
