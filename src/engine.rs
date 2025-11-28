use std::marker::PhantomData;
use std::{path::Path, sync::Arc};

use audio_buffer::SharedSample;
use audio_buffer::symphonia::core::conv::ConvertibleSample;
use audio_buffer::{buffers::interleaved::InterleavedBuffer, loader::error::LoadError};
use audio_graph::AudioGraph;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Sender, TryRecvError, TrySendError};
use time::{FrameTime, SampleRate};

use crate::backend::AudioBackend;
use crate::command::{AudioCommand, Request, Response};
use crate::track::Track;

#[derive(Debug)]
pub enum AudioEngineError {
    QueueFull,
}

pub struct AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    _block_size: FrameTime,
    _sample_rate: SampleRate,
    _bpm: f64,

    sender: Sender<AudioCommand>,
    _stream: Option<cpal::Stream>,
    _marker: PhantomData<T>,
}

impl<T> AudioEngine<T>
where
    T: SharedSample + cpal::SizedSample,
{
    pub fn new(bpm: f64, sample_rate: SampleRate, block_size: FrameTime) -> Self {
        let master_track = Track::from_config(sample_rate, block_size);
        let (graph, master_idx) = AudioGraph::new(master_track, sample_rate, block_size);

        let (sender, receiver) = crossbeam_channel::bounded(256);
        let backend = AudioBackend::new(receiver, graph, master_idx, block_size, bpm, sample_rate);

        let stream = Self::start_stream(backend);

        Self {
            _block_size: block_size,
            _sample_rate: sample_rate,
            _bpm: bpm,
            _stream: Some(stream),
            _marker: PhantomData,
            sender,
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
                    backend.process_block(data);
                    backend.process_commands();
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .expect("failed to build stream");

        stream.play().expect("failed to play stream");
        stream
    }

    pub fn try_send_command(&self, request: Request) -> Result<Response, SendCommandError> {
        let (response_sender, response_receiver) = crossbeam_channel::bounded(1);

        self.sender
            .try_send(AudioCommand {
                response_sender,
                request,
            })
            .map_err(|err| SendCommandError::TrySendError(err))?;

        response_receiver
            .try_recv()
            .map_err(|err| SendCommandError::TryRecvError(err))
    }
}

pub enum SendCommandError {
    TrySendError(TrySendError<AudioCommand>),
    TryRecvError(TryRecvError),
}
