use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Range,
};

use audio_buffer::buffers::interleaved::InterleavedBuffer;
use slotmap::{SlotMap, new_key_type};
use time::{MusicalTime, SampleRate};

new_key_type! { pub struct TrackKey; }
new_key_type! { pub struct BusKey; }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MasterKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendDestination {
    Bus(BusKey),
    Master,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendSource {
    Track(TrackKey),
    Bus(BusKey),
}

pub struct AudioRouting {
    track_sends: HashMap<TrackKey, HashSet<SendDestination>>,
    bus_sends: HashMap<BusKey, HashSet<SendDestination>>,

    // INVARIANT: 'Receives Redundancy'
    // This invariant guarantees that the `bus_receives` aswell as the `master_receives`
    // are synchronized with the `bus_sends` and `track_sends`
    bus_receives: HashMap<BusKey, HashSet<SendSource>>,
    master_receives: HashSet<SendSource>,
}

impl AudioRouting {
    pub fn empty() -> Self {
        Self {
            track_sends: HashMap::new(),
            bus_sends: HashMap::new(),
            bus_receives: HashMap::new(),
            master_receives: HashSet::new(),
        }
    }

    pub fn add_send(&mut self, source: SendSource, destination: SendDestination) {
        match source {
            SendSource::Track(track_key) => {
                self.track_sends
                    .entry(track_key)
                    .or_insert_with(HashSet::new)
                    .insert(destination);
            }
            SendSource::Bus(bus_key) => {
                self.bus_sends
                    .entry(bus_key)
                    .or_insert_with(HashSet::new)
                    .insert(destination);
            }
        }

        match destination {
            SendDestination::Bus(bus) => {
                self.bus_receives
                    .entry(bus)
                    .or_insert_with(HashSet::new)
                    .insert(source);
            }
            SendDestination::Master => {
                self.master_receives.insert(source);
            }
        }
    }

    pub fn remove_send(&mut self, source: SendSource, destination: SendDestination) -> bool {
        let removed = match source {
            SendSource::Track(track_key) => self
                .track_sends
                .get_mut(&track_key)
                .map(|sends| sends.remove(&destination))
                .unwrap_or(false),
            SendSource::Bus(bus_key) => self
                .bus_sends
                .get_mut(&bus_key)
                .map(|sends| sends.remove(&destination))
                .unwrap_or(false),
        };

        if removed {
            match destination {
                SendDestination::Bus(bus_key) => {
                    if let Some(receives) = self.bus_receives.get_mut(&bus_key) {
                        receives.remove(&source);
                    }
                }
                SendDestination::Master => {
                    self.master_receives.remove(&source);
                }
            }
        }

        removed
    }

    /// Remove all sends from this source aswell as potential receives to this source
    // Removes all redundancies aswell
    pub fn remove_source(&mut self, source: SendSource) {
        let destinations = match source {
            SendSource::Track(track_key) => self.track_sends.remove(&track_key),
            SendSource::Bus(bus_key) => {
                self.bus_receives.remove(&bus_key).map(|receives| {
                    for receive in receives {
                        match receive {
                            SendSource::Track(track_key) => self
                                .track_sends
                                .get_mut(&track_key)
                                .map(|sends| sends.remove(&SendDestination::Bus(bus_key))),
                            SendSource::Bus(bus_key) => self
                                .bus_sends
                                .get_mut(&bus_key)
                                .map(|sends| sends.remove(&SendDestination::Bus(bus_key))),
                        };
                    }
                });
                self.bus_sends.remove(&bus_key)
            }
        };

        if let Some(destinations) = destinations {
            for destination in destinations {
                match destination {
                    SendDestination::Bus(bus_key) => {
                        self.bus_receives
                            .get_mut(&bus_key)
                            .map(|receives| receives.remove(&source));
                    }
                    SendDestination::Master => {
                        self.master_receives.remove(&source);
                    }
                };
            }
        }
    }

    pub fn get_sends(&self, source: SendSource) -> Option<&HashSet<SendDestination>> {
        match source {
            SendSource::Track(track_key) => self.track_sends.get(&track_key),
            SendSource::Bus(bus_key) => self.bus_sends.get(&bus_key),
        }
    }

    pub fn get_bus_receives(&self, bus_key: BusKey) -> Option<&HashSet<SendSource>> {
        self.bus_receives.get(&bus_key)
    }

    pub fn get_master_receives(&self) -> &HashSet<SendSource> {
        &self.master_receives
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub sample_rate: SampleRate,
    pub block_range: Range<MusicalTime>,
    pub bpm: f64,
}

pub struct ProcessorConfiguration {
    pub num_input_channels: usize,
    pub num_output_channels: usize,
}

pub trait AudioGenerator<Sample> {
    fn generate(&mut self, output: &mut InterleavedBuffer<Sample>, context: ProcessingContext);

    fn num_channels(&self) -> usize;
}

pub trait AudioTransformer<Sample> {
    fn transform(
        &mut self,
        input: &InterleavedBuffer<Sample>,
        output: &mut InterleavedBuffer<Sample>,
        context: ProcessingContext,
    );

    fn config(&self) -> ProcessorConfiguration;
}

pub struct MixGraph<Sample, Track, Bus> {
    tracks: SlotMap<TrackKey, Track>,
    busses: SlotMap<BusKey, Bus>,

    // INVARIANT: 'Key Validity'
    // This invariant guarantees that all keys inside the `AudioRouting`
    // are valid. This means synchronizing removal of tracks with this
    // structure.
    routing: AudioRouting,
    _marker: PhantomData<Sample>,
}

impl<Sample, Track, Bus> MixGraph<Sample, Track, Bus>
where
    Track: AudioGenerator<Sample>,
    Bus: AudioTransformer<Sample>,
{
    pub fn process_block(
        &mut self,
        output: &mut InterleavedBuffer<Sample>,
        context: ProcessingContext,
    ) {
    }
}

impl<Sample, Track, Bus> MixGraph<Sample, Track, Bus>
where
    Track: Default,
    Bus: Default,
{
    pub fn new_track(&mut self) -> TrackKey {
        self.tracks.insert(Track::default())
    }

    pub fn new_bus(&mut self) -> BusKey {
        self.busses.insert(Bus::default())
    }
}

impl<Sample, Track, Bus> MixGraph<Sample, Track, Bus> {
    pub fn empty() -> Self {
        Self {
            tracks: SlotMap::with_key(),
            busses: SlotMap::with_key(),
            routing: AudioRouting::empty(),
            _marker: PhantomData,
        }
    }

    pub fn insert_track(&mut self, track: Track) -> TrackKey {
        self.tracks.insert(track)
    }

    pub fn insert_bus(&mut self, bus: Bus) -> BusKey {
        self.busses.insert(bus)
    }

    pub fn remove_track(&mut self, track_key: TrackKey) -> Option<Track> {
        let track = self.tracks.remove(track_key);
        self.routing.remove_source(SendSource::Track(track_key));
        track
    }

    pub fn remove_bus(&mut self, bus_key: BusKey) -> Option<Bus> {
        let bus = self.busses.remove(bus_key);
        self.routing.remove_source(SendSource::Bus(bus_key));
        bus
    }

    pub fn add_send(&mut self, source: SendSource, destination: SendDestination) {
        self.routing.add_send(source, destination);
    }

    /// Remove a send from a source. Returns whether removal was performed or the source was dangling
    pub fn remove_send(&mut self, source: SendSource, destination: SendDestination) -> bool {
        self.routing.remove_send(source, destination)
    }

    pub fn get_sends(&self, source: SendSource) -> Option<&HashSet<SendDestination>> {
        self.routing.get_sends(source)
    }

    pub fn get_bus_receives(&self, bus_key: BusKey) -> Option<&HashSet<SendSource>> {
        self.routing.get_bus_receives(bus_key)
    }

    pub fn get_master_receives(&self) -> &HashSet<SendSource> {
        self.routing.get_master_receives()
    }
}
