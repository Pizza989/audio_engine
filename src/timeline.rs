use std::ops::Range;

use interavl::IntervalTree;
use time::{FrameTime, MusicalTime, SampleRate};

use crate::audio::cache::BufferKey;

#[derive(Clone, Debug)]
pub struct Event {
    pub buffer_slice_range: Range<FrameTime>,
    pub buffer: BufferKey,
}

pub struct Clip {
    pub buffer: BufferKey,
    // The offset into the buffer
    pub offset: FrameTime,
}

#[derive(Debug)]
pub struct BlockEvent {
    pub offset: FrameTime,
    pub event: Event,
}

fn get_intersection(
    block_range: &Range<MusicalTime>,
    clip_range: &Range<MusicalTime>,
) -> Option<Range<MusicalTime>> {
    let start = block_range.start.max(clip_range.start);
    let end = block_range.end.min(clip_range.end);

    if start < end { Some(start..end) } else { None }
}

pub struct Timeline {
    bpm: f64,
    sample_rate: SampleRate,
    clips: IntervalTree<MusicalTime, Clip>,
}

impl Timeline {
    pub fn new(bpm: f64, sample_rate: SampleRate, clips: IntervalTree<MusicalTime, Clip>) -> Self {
        Self {
            bpm,
            sample_rate,
            clips,
        }
    }

    // overwrites anything that was at this range previously
    pub fn insert(&mut self, range: Range<MusicalTime>, clip: Clip) -> Result<(), ()> {
        if range.start >= range.end {
            Err(())
        } else {
            self.clips.insert(range, clip);
            Ok(())
        }
    }

    pub fn iter_blocks(&self, block_size: FrameTime) -> impl Iterator<Item = Vec<BlockEvent>> {
        (0..).map(move |block_idx| {
            let start = block_size * (block_idx as u64);
            let end = start + block_size;

            // WARNING lossy conversion i have no clue if this can be implemented
            // without one
            let block_range = start.to_musical_lossy(self.bpm, self.sample_rate)
                ..end.to_musical_lossy(self.bpm, self.sample_rate);

            let mut block_events = Vec::new();
            for (clip_range, clip) in self.clips.iter_overlaps(&block_range) {
                // safe unwrap because they are known to overlap
                let intersection = get_intersection(&block_range, &clip_range).unwrap();
                // safe unwrap because intersection is known to be inside of clip_range
                // therefore intersection.start >= clip_range.start
                let clip_local_intersection =
                    (intersection.start.checked_sub(clip_range.start).unwrap())
                        ..(intersection.end.checked_sub(clip_range.start).unwrap());

                // WARNING lossy conversion i have no clue if this can be implemented
                // without one
                let buffer_slice_range = clip_local_intersection
                    .start
                    .to_nearest_frame_round_lossy(self.bpm, self.sample_rate)
                    ..clip_local_intersection
                        .end
                        .to_nearest_frame_round_lossy(self.bpm, self.sample_rate);

                let event = Event {
                    buffer_slice_range: buffer_slice_range.clone(),
                    buffer: clip.buffer,
                };

                // OVERFLOW: because of the lossy conversions in the case where
                // `start` should be equal to `buffer_slice_range.start` it can
                // happen that it is actually bigger which would cause an over-
                // flow because FrameTime is a wrapper around u64 which cannot
                // be negative
                let block_event = if start > buffer_slice_range.start {
                    BlockEvent {
                        offset: FrameTime::new(0),
                        event,
                    }
                } else {
                    BlockEvent {
                        offset: buffer_slice_range.start - start,
                        event,
                    }
                };
                block_events.push(block_event);
            }

            block_events
        })
    }
}
