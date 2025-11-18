use std::marker::PhantomData;
use std::{path::Path, sync::Arc};

use audio_buffer::SharedSample;
use audio_buffer::symphonia::core::conv::ConvertibleSample;
use audio_buffer::{buffers::interleaved::InterleavedBuffer, loader::error::LoadError};
use audio_graph::AudioGraph;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use ringbuf::traits::Split;
use time::{FrameTime, SampleRate};

use crate::backend::AudioBackend;
use crate::message::{AudioBackendMessage, AudioEngineMessage};
use crate::track::Track;

pub struct AudioEngine<T>
where
    T: audio_buffer::dasp::Sample + 'static,
{
    _block_size: FrameTime,
    _sample_rate: SampleRate,
    _bpm: f64,

    _stream: Option<cpal::Stream>,
    _marker: PhantomData<T>,
}

impl<T> AudioEngine<T>
where
    T: SharedSample + cpal::SizedSample,
{
    pub fn new(bpm: f64, sample_rate: SampleRate, block_size: FrameTime) -> Self {
        let (_cmd_prod, cmd_cons) = HeapRb::<AudioBackendMessage>::new(256).split();
        let (status_prod, _status_cons) = HeapRb::<AudioEngineMessage>::new(256).split();

        let master_track = Track::from_config(sample_rate, block_size);
        let (graph, master_idx) = AudioGraph::new(master_track, sample_rate, block_size);

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
                    backend.process_commands();
                    backend.process_block(data);
                },
                |err| eprintln!("Stream error: {}", err),
                None,
            )
            .expect("failed to build stream");

        stream.play().expect("failed to play stream");
        stream
    }

    pub fn load_audio_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<Arc<InterleavedBuffer<T>>, LoadError>
    where
        T: ConvertibleSample,
    {
        audio_buffer::loader::load(path).map(|buffer| Arc::new(buffer))
    }
}
