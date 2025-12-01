pub use dasp;
pub use symphonia;

// TODO: These are premature. For now just use one AudioBuffer with one memory layout
// that is more conveniet: `AudioBuffer`
// pub mod buffers;
// pub mod core;

// TODO: rewrite with new audio buffer
// #[cfg(feature = "loader")]
// pub mod loader;

pub trait SharedSample: dasp::Sample + Send + Sync + 'static {}
impl<T> SharedSample for T where T: dasp::Sample + Send + Sync + 'static {}

// Strategy for handling channel count mismatches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStrategy {
    /// Take only the minimum number of channels available
    Min,
    /// Use all channels from source, repeating if destination has more
    Repeat,
    /// Mix down source channels if destination has fewer, upmix if more
    MixAdapt,
    /// Strict - fail if channel counts don't match
    Strict,
}

// Strategy for handling frame count mismatches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStrategy {
    /// Copy only the minimum number of frames available
    Min,
    /// Truncate or zero-pad to fit destination
    Fit,
    /// Strict - fail if frame counts don't match
    Strict,
}

#[derive(Debug, Clone, Copy)]
pub struct CopyOptions {
    pub channel_strategy: ChannelStrategy,
    pub frame_strategy: FrameStrategy,
    pub src_channel_offset: usize,
    pub dst_channel_offset: usize,
    pub src_frame_offset: usize,
    pub dst_frame_offset: usize,
}

impl CopyOptions {
    pub fn with_src_frame_offset(mut self, offset: usize) -> Self {
        self.src_frame_offset = offset;
        self
    }
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            channel_strategy: ChannelStrategy::Min,
            frame_strategy: FrameStrategy::Min,
            src_channel_offset: 0,
            dst_channel_offset: 0,
            src_frame_offset: 0,
            dst_frame_offset: 0,
        }
    }
}

pub struct AudioBuffer<S>
where
    S: dasp::Sample,
{
    num_channels: usize,
    num_frames: usize,
    data: Vec<S>,
}

impl<S> AudioBuffer<S>
where
    S: dasp::Sample,
{
    pub fn new() -> Self {
        Self {
            data: vec![],
            num_channels: 0,
            num_frames: 0,
        }
    }

    pub fn with_shape(num_channels: usize, num_frames: usize) -> Self {
        Self {
            data: vec![S::EQUILIBRIUM; num_channels * num_frames],
            num_channels,
            num_frames,
        }
    }

    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    pub fn num_frames(&self) -> usize {
        self.num_frames
    }

    pub fn get_channel(&self, index: usize) -> &[S] {
        let start = index * self.num_frames;
        &self.data[start..start + self.num_frames]
    }

    pub fn get_channel_mut(&mut self, index: usize) -> &mut [S] {
        let start = index * self.num_frames;
        &mut self.data[start..start + self.num_frames]
    }
}

impl<S> AudioBuffer<S>
where
    S: dasp::Sample + Clone,
{
    /// Copy from another buffer with configurable strategies
    ///
    /// Returns Err when an offset is out of bounds, or when
    /// ChannelStrategy or FrameStragey is in Strict mode and
    /// there is a mismatch with either of them
    pub fn copy_from(
        &mut self,
        source: &AudioBuffer<S>,
        options: CopyOptions,
    ) -> Result<(), BufferError> {
        // Validate offsets
        if options.src_channel_offset >= source.num_channels {
            return Err(BufferError::InvalidChannelOffset);
        }
        if options.dst_channel_offset >= self.num_channels {
            return Err(BufferError::InvalidChannelOffset);
        }
        if options.src_frame_offset >= source.num_frames {
            return Err(BufferError::InvalidFrameOffset);
        }
        if options.dst_frame_offset >= self.num_frames {
            return Err(BufferError::InvalidFrameOffset);
        }

        let src_channels_available = source.num_channels - options.src_channel_offset;
        let dst_channels_available = self.num_channels - options.dst_channel_offset;

        let channels_to_copy = match options.channel_strategy {
            ChannelStrategy::Min => src_channels_available.min(dst_channels_available),
            ChannelStrategy::Repeat => dst_channels_available,
            ChannelStrategy::MixAdapt => dst_channels_available,
            ChannelStrategy::Strict => {
                if src_channels_available != dst_channels_available {
                    return Err(BufferError::ChannelMismatch);
                }
                src_channels_available
            }
        };

        let src_frames_available = source.num_frames - options.src_frame_offset;
        let dst_frames_available = self.num_frames - options.dst_frame_offset;

        let frames_to_copy = match options.frame_strategy {
            FrameStrategy::Min => src_frames_available.min(dst_frames_available),
            FrameStrategy::Fit => dst_frames_available,
            FrameStrategy::Strict => {
                if src_frames_available != dst_frames_available {
                    return Err(BufferError::FrameMismatch);
                }
                src_frames_available
            }
        };

        for dst_ch in 0..channels_to_copy {
            let actual_dst_ch = dst_ch + options.dst_channel_offset;

            let src_ch = match options.channel_strategy {
                ChannelStrategy::Repeat => {
                    (dst_ch % src_channels_available) + options.src_channel_offset
                }
                _ => dst_ch + options.src_channel_offset,
            };

            let copy_len = frames_to_copy.min(src_frames_available);
            let src_slice = &source.get_channel(src_ch)[options.src_frame_offset..][..copy_len];
            let dst_slice =
                &mut self.get_channel_mut(actual_dst_ch)[options.dst_frame_offset..][..copy_len];

            dst_slice.copy_from_slice(src_slice);

            // Handle padding if FrameStrategy::Fit and destination is larger
            if matches!(options.frame_strategy, FrameStrategy::Fit)
                && dst_frames_available > src_frames_available
            {
                let pad_start = options.dst_frame_offset + copy_len;
                let pad_end = options.dst_frame_offset + dst_frames_available;
                let dst_channel = self.get_channel_mut(actual_dst_ch);
                for frame in &mut dst_channel[pad_start..pad_end] {
                    *frame = S::EQUILIBRIUM;
                }
            }
        }

        Ok(())
    }

    /// Mix from another buffer with configurable strategies
    ///
    /// Returns Err when the ChannelStrategy is Strict and there is a channel-mismatch
    pub fn mix_from(
        &mut self,
        source: &AudioBuffer<S>,
        gain: S::Float,
        options: CopyOptions,
    ) -> Result<(), BufferError>
    where
        S::Float: dasp::Sample,
    {
        let src_channels_available = source.num_channels - options.src_channel_offset;
        let dst_channels_available = self.num_channels - options.dst_channel_offset;

        let channels_to_mix = match options.channel_strategy {
            ChannelStrategy::Strict if src_channels_available != dst_channels_available => {
                return Err(BufferError::ChannelMismatch);
            }
            _ => src_channels_available.min(dst_channels_available),
        };

        let frames_to_mix = (source.num_frames - options.src_frame_offset)
            .min(self.num_frames - options.dst_frame_offset);

        for ch in 0..channels_to_mix {
            let src_ch = ch + options.src_channel_offset;
            let dst_ch = ch + options.dst_channel_offset;

            let src_slice =
                &source.get_channel(src_ch)[options.src_frame_offset..][..frames_to_mix];
            let dst_slice =
                &mut self.get_channel_mut(dst_ch)[options.dst_frame_offset..][..frames_to_mix];

            for (dst_sample, src_sample) in dst_slice.iter_mut().zip(src_slice.iter()) {
                *dst_sample = dst_sample.add_amp(src_sample.mul_amp(gain).to_signed_sample());
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.data.fill(S::EQUILIBRIUM);
    }

    /// Resize the buffer, preserving existing data where possible
    pub fn resize(&mut self, num_channels: usize, num_frames: usize) {
        let new_size = num_channels * num_frames;
        let mut new_data = vec![S::EQUILIBRIUM; new_size];

        let channels_to_copy = self.num_channels.min(num_channels);
        let frames_to_copy = self.num_frames.min(num_frames);

        for ch in 0..channels_to_copy {
            let old_start = ch * self.num_frames;
            let new_start = ch * num_frames;

            new_data[new_start..new_start + frames_to_copy]
                .copy_from_slice(&self.data[old_start..old_start + frames_to_copy]);
        }

        self.data = new_data;
        self.num_channels = num_channels;
        self.num_frames = num_frames;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    ChannelMismatch,
    FrameMismatch,
    InvalidChannelOffset,
    InvalidFrameOffset,
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::ChannelMismatch => write!(f, "Channel count mismatch"),
            BufferError::FrameMismatch => write!(f, "Frame count mismatch"),
            BufferError::InvalidChannelOffset => write!(f, "Invalid channel offset"),
            BufferError::InvalidFrameOffset => write!(f, "Invalid frame offset"),
        }
    }
}

impl std::error::Error for BufferError {}
