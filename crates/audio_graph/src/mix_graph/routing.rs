use slotmap::SlotMap;

use crate::{
    mix_graph::{
        BusKey, Connection, ConnectionDestination, ConnectionKey, ConnectionSource, TrackKey,
    },
    pin_matrix::PinMatrix,
};

// TODO:
// - cycle detection
pub struct AudioRouting {
    connections: SlotMap<ConnectionKey, Connection>,
}

impl AudioRouting {
    pub fn empty() -> Self {
        Self {
            connections: SlotMap::with_key(),
        }
    }

    // Whether the matrix is valid is not checked here
    pub fn add_connection(
        &mut self,
        source: ConnectionSource,
        destination: ConnectionDestination,
        matrix: PinMatrix,
    ) -> ConnectionKey {
        self.connections.insert(Connection {
            source,
            destination,
            matrix,
        })
    }

    pub fn remove_connection(&mut self, key: ConnectionKey) -> Option<Connection> {
        self.connections.remove(key)
    }

    pub fn remove_track(&mut self, track_key: TrackKey) {
        self.connections
            .retain(|_, connection| connection.source != ConnectionSource::Track(track_key));
    }

    pub fn remove_bus(&mut self, bus_key: BusKey) {
        self.connections.retain(|_, connection| {
            connection.source != ConnectionSource::Bus(bus_key)
                && connection.destination != ConnectionDestination::Bus(bus_key)
        });
    }

    pub fn get_sends(
        &self,
        source: ConnectionSource,
    ) -> impl Iterator<Item = (ConnectionKey, &Connection)> {
        self.connections
            .iter()
            .filter(move |(_, connection)| connection.source == source)
    }

    pub fn get_receives(
        &self,
        destination: ConnectionDestination,
    ) -> impl Iterator<Item = (ConnectionKey, &Connection)> {
        self.connections
            .iter()
            .filter(move |(_, connection)| connection.destination == destination)
    }
}
