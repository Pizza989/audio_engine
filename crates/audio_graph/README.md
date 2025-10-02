# Things i don't like about the current design
- node indices can dangle. this is account for by mapping those cases to GraphError::InvalidNode
- connections in the dag must be valid, which means: `src.output_channels == dst.input_channels`
  this is enforced at runtime
- currently the buffers in use are backed by a Vec so they will have to be grown on first use or
  initialized with the correct capacity. a potential upgrade would be the FixedFramesBuffer from
  the audio_buffer crate if that is ever finished
- process_from returns a Result because it calls AudioProcessor::process which returns a result.
  however it can be assured by writing correct code that process will not return an Err variant
  therefore process_unchecked could be used.
