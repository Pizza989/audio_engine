use audio_buffer::SharedSample;
use audio_graph::AudioGraph;
use audio_graph::daggy::{Dag, EdgeIndex, NodeIndex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use time::{FrameTime, SampleRate};

use crate::adjacency_matrix::AdjacencyMatrix;
use crate::backend::AudioBackend;
use crate::message::{
    AudioBackendCommand, AudioBackendMessage, AudioEngineMessage, AudioEngineStatus, MessageId,
};
use crate::track::Track;

#[derive(Debug)]
pub enum AudioEngineError {
    QueueFull,
}

pub struct EmptyWeight;
pub type StructuralGraph = Dag<EmptyWeight, EmptyWeight>;

pub struct AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    _block_size: FrameTime,
    _sample_rate: SampleRate,
    _bpm: f64,
    // A structurally indentical version of the audio graph owned by the backend
    // INVARIANT: must be up to date with the backend's graph
    adjacency_matrix: AdjacencyMatrix,

    next_message_id: u64,
    command_producer: HeapProd<AudioBackendMessage>,
    status_consumer: HeapCons<AudioEngineMessage>,
    status_message_cache: VecDeque<AudioEngineMessage>,
    _stream: Option<cpal::Stream>,
    _marker: PhantomData<T>,
}

impl<T> AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    /// Get a reference to a HashSet of NodeIndices
    ///
    /// # PRECONDITIONS
    /// User must make sure the graph is updated with the {} method
    // TODO: update docs when there is a unified update method
    pub fn nodes(&mut self) -> &HashSet<NodeIndex> {
        self.adjacency_matrix.nodes()
    }

    /// Get a reference to a map from EdgeIndex to edge
    ///
    /// # PRECONDITIONS
    /// User must make sure the graph is updated with the {} method
    // TODO: update docs when there is a unified update method
    pub fn edges(&mut self) -> &HashMap<EdgeIndex, (NodeIndex, NodeIndex)> {
        self.adjacency_matrix.edges()
    }

    pub fn update_adjacency_matrix(&mut self) {
        while let Some(message) = self.status_consumer.try_pop() {
            match message.status {
                AudioEngineStatus::RemoveNode(node_index) => todo!(),
                AudioEngineStatus::AddEdge {
                    index,
                    source,
                    destination,
                } => todo!(),
                AudioEngineStatus::RemoveEdge(edge_index) => todo!(),
                AudioEngineStatus::AddNode(node_index) => todo!(),
            }
        }
    }
}

impl<T> AudioEngine<T>
where
    T: SharedSample + cpal::SizedSample,
{
    pub fn new(bpm: f64, sample_rate: SampleRate, block_size: FrameTime) -> Self {
        let (cmd_prod, cmd_cons) = HeapRb::<AudioBackendMessage>::new(256).split();
        let (status_prod, status_cons) = HeapRb::<AudioEngineMessage>::new(256).split();

        let master_track = Track::from_config(sample_rate, block_size);
        let (graph, master_idx) = AudioGraph::new(master_track, sample_rate, block_size);

        let mut adjacency_matrix = AdjacencyMatrix::empty();
        adjacency_matrix.add_node(master_idx);

        let backend = AudioBackend::new(
            cmd_cons,
            status_prod,
            graph,
            master_idx,
            block_size,
            bpm,
            sample_rate,
        );

        let stream = Self::start_stream(backend);

        Self {
            _block_size: block_size,
            _sample_rate: sample_rate,
            _bpm: bpm,
            _stream: Some(stream),
            _marker: PhantomData,
            command_producer: cmd_prod,
            status_consumer: status_cons,
            next_message_id: 0,
            status_message_cache: VecDeque::new(),
            adjacency_matrix,
        }
    }

    fn start_stream(mut backend: AudioBackend<T>) -> cpal::Stream {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let config = device.default_output_config().unwrap();

        let stream = device
            .build_output_stream(
                &config.config(),
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    backend.process_messages();
                    backend.process_block(data);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .expect("failed to build stream");

        stream.play().expect("failed to play stream");
        stream
    }

    fn next_message_id(&mut self) -> MessageId {
        let id = self.next_message_id;
        self.next_message_id = self.next_message_id.wrapping_add(1);
        MessageId(id)
    }

    fn new_message(&mut self, command: AudioBackendCommand) -> AudioBackendMessage {
        AudioBackendMessage {
            id: self.next_message_id(),
            command,
        }
    }

    pub fn dispatch_command(
        &mut self,
        command: AudioBackendCommand,
    ) -> Result<(), AudioEngineError> {
        let message = self.new_message(command);

        self.command_producer
            .try_push(message)
            .map_err(|_| AudioEngineError::QueueFull)?;

        Ok(())
    }
}
