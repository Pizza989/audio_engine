use std::path::Path;

use symphonia::core::{audio::SampleBuffer, codecs::DecoderOptions, conv::ConvertibleSample};

use crate::{
    buffers::{compatability::slice::WrapInterleaved, interleaved::InterleavedBuffer},
    core::io::writer::Writer,
    loader::error::LoadError,
};

pub mod error;
pub mod probe;

pub fn load<T: ConvertibleSample + dasp::Sample + 'static>(
    path: impl AsRef<Path>,
) -> Result<InterleavedBuffer<T>, LoadError> {
    let source = probe::probe_file(path, None)?;

    let mut format = source.probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| LoadError::NoTrackFound)?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| LoadError::CouldNotCreateDecoder(e))?;

    let track_id = track.id;
    let mut sample_buffer = None;

    let mut final_buffer = InterleavedBuffer::<T>::with_capacity(
        source.num_channels,
        source.sample_rate.into(),
        track.codec_params.n_frames.unwrap_or(0) as usize,
    );
    let mut writer = Writer::new(&mut final_buffer);

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buffer) => {
                if sample_buffer.is_none() {
                    sample_buffer = Some(SampleBuffer::<T>::new(
                        audio_buffer.capacity() as u64,
                        *audio_buffer.spec(),
                    ));
                }

                if let Some(buffer) = &mut sample_buffer {
                    buffer.copy_interleaved_ref(audio_buffer);
                    let compat = WrapInterleaved::new(
                        buffer.samples(),
                        source.num_channels.get(),
                        source.sample_rate.into(),
                    );
                    writer.write_block_growing(&compat)?;
                }
            }
            Err(e) => {
                return Err(LoadError::ErrorWhileDecoding(e));
            }
        }
    }

    Ok(final_buffer)
}
