use crate::audio::error::LoadError;
use std::path::Path;
use std::{fs::File, num::NonZeroUsize};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, Probe, ProbeResult};

pub struct LoadedAudioSource {
    pub probed: ProbeResult,
    pub sample_rate: u32,
    pub num_channels: NonZeroUsize,
}

pub fn probe_file<P: AsRef<Path>>(
    path: P,
    probe: &'static Probe,
) -> Result<LoadedAudioSource, LoadError> {
    let path: &Path = path.as_ref();

    let file = File::open(path)?;
    let mut hint = Hint::new();

    if let Some(extension) = path.extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }

    probe_audio_source(Box::new(file), Some(hint), probe)
}

pub fn probe_audio_source(
    source: Box<dyn MediaSource>,
    hint: Option<Hint>,
    probe: &'static Probe,
) -> Result<LoadedAudioSource, LoadError> {
    let mss = MediaSourceStream::new(source, Default::default());

    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();

    let hint = hint.unwrap_or_default();

    let probed = probe
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| LoadError::UnkownFormat(e))?;

    let track = probed
        .format
        .default_track()
        .ok_or_else(|| LoadError::NoTrackFound)?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or_else(|| {
        // log::warn!("Could not find sample rate of PCM resource. Assuming a sample rate of 44100");
        44100
    });

    let num_channels = track
        .codec_params
        .channels
        .ok_or_else(|| LoadError::NoChannelsFound)?
        .count();

    if num_channels == 0 {
        return Err(LoadError::NoChannelsFound);
    }

    Ok(LoadedAudioSource {
        probed,
        sample_rate,
        num_channels: NonZeroUsize::new(num_channels).unwrap(),
    })
}
