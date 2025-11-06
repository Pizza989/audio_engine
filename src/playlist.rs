use std::ops::Range;

use interavl::IntervalTree;
use time::{FrameTime, MusicalTime, SampleRate};

use crate::memory::BufferKey;

#[derive(Debug, Clone)]
pub struct Event {
    pub buffer_slice_range: Range<FrameTime>,
    pub buffer: BufferKey,
}

#[derive(Debug, Clone, Copy)]
pub struct Clip {
    pub buffer_key: BufferKey,
    pub buffer_offset: FrameTime,
}

#[derive(Debug, Clone)]
pub struct BlockEvent {
    pub block_offset: FrameTime,
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

pub struct Playlist {
    clips: IntervalTree<MusicalTime, Clip>,
}

impl Playlist {
    pub fn from_clips(clips: IntervalTree<MusicalTime, Clip>) -> Self {
        Self { clips }
    }

    pub fn empty() -> Self {
        Self {
            clips: IntervalTree::default(),
        }
    }
}

impl Playlist {
    /// Insert a `Clip` into the `Playlist`
    /// Returns the previously existing clip at this range, or `None` if there wasn't any
    ///
    /// # Panics
    /// Panics if `range.start >= range.end`
    pub fn insert(&mut self, range: Range<MusicalTime>, clip: Clip) -> Option<Clip> {
        assert!(
            range.start < range.end,
            "invalid range: start must be less than end"
        );
        self.clips.insert(range, clip)
    }

    pub fn remove(&mut self, range: Range<MusicalTime>) -> Option<Clip> {
        self.clips.remove(&range)
    }

    pub fn get(&self, range: Range<MusicalTime>) -> Option<Clip> {
        self.clips.get(&range).copied()
    }

    pub fn iter_blocks(
        &self,
        block_size: FrameTime,
        sample_rate: SampleRate,
        bpm: f64,
    ) -> BlockIterator<'_> {
        let block_duration_musical = block_size.to_musical_lossy(bpm, sample_rate);

        BlockIterator {
            bpm,
            sample_rate,
            current_musical_pos: MusicalTime::ZERO,
            block_duration_musical,
            clips: &self.clips,
        }
    }
}

/// An Iterator that generates BlockEvents from an IntervalTree
pub struct BlockIterator<'a> {
    bpm: f64,
    sample_rate: SampleRate,
    current_musical_pos: MusicalTime,
    block_duration_musical: MusicalTime,
    clips: &'a IntervalTree<MusicalTime, Clip>,
}

impl Iterator for BlockIterator<'_> {
    type Item = Vec<BlockEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let block_start_musical = self.current_musical_pos;
        let block_end_musical = block_start_musical + self.block_duration_musical;
        let block_range_musical = block_start_musical..block_end_musical;

        let mut block_events = Vec::new();

        for (clip_range, clip) in self.clips.iter_overlaps(&block_range_musical) {
            let intersection = match get_intersection(&block_range_musical, &clip_range) {
                Some(i) => i,
                None => continue,
            };

            let clip_local_start = intersection.start.checked_sub(clip_range.start).unwrap();
            let clip_local_end = intersection.end.checked_sub(clip_range.start).unwrap();

            let buffer_start_frame =
                clip_local_start.to_nearest_frame_round_lossy(self.bpm, self.sample_rate);
            let buffer_end_frame =
                clip_local_end.to_nearest_frame_round_lossy(self.bpm, self.sample_rate);

            let buffer_slice_start = clip.buffer_offset + buffer_start_frame;
            let buffer_slice_end = clip.buffer_offset + buffer_end_frame;

            let event = Event {
                buffer_slice_range: buffer_slice_start..buffer_slice_end,
                buffer: clip.buffer_key,
            };

            let offset_musical = intersection.start.checked_sub(block_start_musical).unwrap();
            let offset_frames =
                offset_musical.to_nearest_frame_round_lossy(self.bpm, self.sample_rate);

            block_events.push(BlockEvent {
                block_offset: offset_frames,
                event,
            });
        }

        self.current_musical_pos = block_end_musical;

        Some(block_events)
    }
}
