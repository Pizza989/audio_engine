use std::marker::PhantomData;

use audio_buffer::AudioBuffer;
use slotmap::{SlotMap, new_key_type};

use crate::{
    mix_graph::{
        processor::{AudioGenerator, AudioTransformer, ProcessingContext},
        routing::AudioRouting,
    },
    pin_matrix::PinMatrix,
};

pub mod processor;
pub mod routing;
pub mod transformer_chain;

new_key_type! { pub struct TrackKey; }
new_key_type! { pub struct BusKey; }
new_key_type! { pub struct ConnectionKey; }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MasterKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionSource {
    Track(TrackKey),
    Bus(BusKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionDestination {
    Bus(BusKey),
    Master,
}

pub struct Connection {
    source: ConnectionSource,
    destination: ConnectionDestination,
    matrix: PinMatrix,
}

pub struct MixGraph<Sample, Track, Bus> {
    tracks: SlotMap<TrackKey, Track>,
    busses: SlotMap<BusKey, Bus>,
    master_bus: Bus,

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
    Sample: audio_buffer::dasp::Sample,
{
    pub fn process_block(&mut self, output: &mut AudioBuffer<Sample>, context: ProcessingContext) {}
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
    pub fn new(master_bus: Bus) -> Self {
        Self {
            tracks: SlotMap::with_key(),
            busses: SlotMap::with_key(),
            routing: AudioRouting::empty(),
            _marker: PhantomData,
            master_bus,
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
        self.routing.remove_track(track_key);
        track
    }

    pub fn remove_bus(&mut self, bus_key: BusKey) -> Option<Bus> {
        let bus = self.busses.remove(bus_key);
        self.routing.remove_bus(bus_key);
        bus
    }

    // TODO: check matrix validity
    pub fn add_send(
        &mut self,
        source: ConnectionSource,
        destination: ConnectionDestination,
        matrix: PinMatrix,
    ) {
        self.routing.add_connection(source, destination, matrix);
    }

    /// Remove a send from a source. Returns whether removal was performed or the source was dangling
    pub fn remove_send(&mut self, key: ConnectionKey) -> Option<Connection> {
        self.routing.remove_connection(key)
    }

    pub fn get_sends(
        &self,
        source: ConnectionSource,
    ) -> impl Iterator<Item = (ConnectionKey, &Connection)> {
        self.routing.get_sends(source)
    }

    pub fn get_receives(
        &self,
        destination: ConnectionDestination,
    ) -> impl Iterator<Item = (ConnectionKey, &Connection)> {
        self.routing.get_receives(destination)
    }
}
