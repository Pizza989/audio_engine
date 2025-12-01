use std::{ops::Range, sync::Arc};

use audio_buffer::{AudioBuffer, dasp};
use interavl::IntervalTree;
use time::{FrameTime, MusicalTime, SampleRate};

#[derive(Clone)]
pub struct BlockEvent<T: dasp::Sample> {
    pub block_offset: usize,
    pub event: Event<T>,
}

#[derive(Clone)]
pub struct Event<T: dasp::Sample> {
    pub buffer: Arc<AudioBuffer<T>>,
}

pub struct Clip<T: dasp::Sample> {
    pub buffer: Arc<AudioBuffer<T>>,
}

impl<T> Clone for Clip<T>
where
    T: dasp::Sample,
{
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

pub struct Playlist<T>
where
    T: dasp::Sample,
{
    clips: IntervalTree<MusicalTime, Clip<T>>,
}

impl<T> Playlist<T>
where
    T: dasp::Sample,
{
    pub fn from_clips(clips: IntervalTree<MusicalTime, Clip<T>>) -> Self {
        Self { clips }
    }

    pub fn empty() -> Self {
        Self {
            clips: IntervalTree::default(),
        }
    }
}

impl<T> Playlist<T>
where
    T: dasp::Sample,
{
    /// Insert a `Clip` into the `Playlist`
    /// Returns the previously existing clip at this range, or `None` if there wasn't any
    ///
    /// # Panics
    /// Panics if `range.start >= range.end`
    pub fn insert(&mut self, range: Range<MusicalTime>, clip: Clip<T>) -> Option<Clip<T>> {
        assert!(
            range.start < range.end,
            "invalid range: start must be less than end"
        );
        self.clips.insert(range, clip)
    }

    pub fn remove(&mut self, range: Range<MusicalTime>) -> Option<Clip<T>> {
        self.clips.remove(&range)
    }

    pub fn get(&self, range: Range<MusicalTime>) -> Option<Clip<T>> {
        self.clips.get(&range).cloned()
    }

    // TODO: currently not needed; maybe remove?
    pub fn iter_blocks(
        &self,
        block_size: FrameTime,
        sample_rate: SampleRate,
        bpm: f64,
    ) -> BlockIterator<'_, T> {
        let block_duration_musical = block_size.to_musical_lossy(bpm, sample_rate);

        BlockIterator {
            bpm,
            sample_rate,
            current_musical_pos: MusicalTime::ZERO,
            block_duration_musical,
            clips: &self.clips,
        }
    }

    pub fn get_block_events(
        &self,
        block_range_musical: Range<MusicalTime>,
        bpm: f64,
        sample_rate: SampleRate,
    ) -> Vec<BlockEvent<T>> {
        let mut block_events = Vec::new();

        for (clip_range, clip) in self.clips.iter_overlaps(&block_range_musical) {
            let event = Event {
                buffer: clip.buffer.clone(),
            };

            let offset_musical = clip_range
                .start
                .checked_sub(block_range_musical.start)
                .unwrap_or(MusicalTime::ZERO);

            let offset_frames = offset_musical.to_nearest_frame_round_lossy(bpm, sample_rate);

            block_events.push(BlockEvent {
                block_offset: offset_frames.0 as usize,
                event,
            });
        }

        block_events
    }
}

/// An Iterator that generates BlockEvents from an IntervalTree
// TODO: currently not needed; maybe remove?
pub struct BlockIterator<'a, T>
where
    T: dasp::Sample,
{
    bpm: f64,
    sample_rate: SampleRate,
    current_musical_pos: MusicalTime,
    block_duration_musical: MusicalTime,
    clips: &'a IntervalTree<MusicalTime, Clip<T>>,
}

impl<T> Iterator for BlockIterator<'_, T>
where
    T: dasp::Sample,
{
    type Item = Vec<BlockEvent<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let block_start_musical = self.current_musical_pos;
        let block_end_musical = block_start_musical + self.block_duration_musical;
        let block_range_musical = block_start_musical..block_end_musical;

        let mut block_events = Vec::new();

        for (clip_range, clip) in self.clips.iter_overlaps(&block_range_musical) {
            let event = Event {
                buffer: clip.buffer.clone(),
            };

            let offset_musical = clip_range
                .start
                .checked_sub(block_start_musical)
                .unwrap_or(MusicalTime::ZERO);

            let offset_frames =
                offset_musical.to_nearest_frame_round_lossy(self.bpm, self.sample_rate);

            block_events.push(BlockEvent {
                block_offset: offset_frames.0 as usize,
                event,
            });
        }

        self.current_musical_pos = block_end_musical;

        Some(block_events)
    }
}
