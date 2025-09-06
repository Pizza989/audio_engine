use std::collections::BTreeMap;

use time::{FrameTime, MusicalTime, SampleRate};

#[derive(Clone, Debug)]
enum Payload {
    Midi,
    Audio,
}

#[derive(Clone, Debug)]
pub struct Event {
    payload: Payload,
}

#[derive(Debug)]
pub struct BlockEvent {
    offset: FrameTime,
    event: Event,
}

pub struct Timeline {
    bpm: f64,
    sample_rate: SampleRate,
    events: BTreeMap<MusicalTime, Vec<Event>>,
}

impl Timeline {
    pub fn new(
        bpm: f64,
        sample_rate: SampleRate,
        events: BTreeMap<MusicalTime, Vec<Event>>,
    ) -> Self {
        Self {
            bpm,
            sample_rate,
            events,
        }
    }

    pub fn insert(&mut self, start: MusicalTime, event: Event) {
        match self.events.get_mut(&start) {
            Some(events) => events.push(event),
            None => {
                self.events.insert(start, vec![event]);
            }
        };
    }

    pub fn iter_blocks(&self, block_size: FrameTime) -> impl Iterator<Item = Vec<BlockEvent>> {
        (0..).map(move |block_idx| {
            let start = block_size * (block_idx as u64);
            let end = start + block_size;

            // WARNING: this may produce incorrect events due to lossy
            // conversions (maybe). what could happen then is that
            // events that are n samples outside the range are still
            // included or vice versa. in theory this could of course
            // be implemented lossless however i don't posses the mental
            // capacity as of right now.
            self.events
                .range(
                    start.to_musical_lossy(self.bpm, self.sample_rate)
                        ..end.to_musical_lossy(self.bpm, self.sample_rate),
                )
                .flat_map(|(musical_time, events)| {
                    events.iter().map(move |ev| BlockEvent {
                        offset: musical_time
                            .to_nearest_frame_round_lossy(self.bpm, self.sample_rate)
                            - start,
                        event: ev.clone(),
                    })
                })
                .collect::<Vec<_>>()
        })
    }
}
