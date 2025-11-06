# Links
https://github.com/MeadowlarkDAW/meadowlark-core-types/blob/main/src/time/superclock_time.rs

# TODO
- implement run method on AudioEngine
  - figure out how to handle input and output buffers in
    audio graphs that have sub graphs
      - probably means having to add multi input support
        to AudioGraph

  - create a buffer storage for loaded audio clips on the
    AudioEngine

  - extend timeline to work 2d and associate clips with time
    and track
    - allow indexing of the tracks
  
- add reconfiguration to audio_graph
  - create a newtype wrapper that stores the channel amounts
    and the processor. that way the processor can be handed
    out without allowing reconfiguration to pass the invariants
    of the AudioGraph

- how to handle sample rate of low level buffers in the audio
  graph. The InterleavedBuffer stores a sample rate. however
  in the audio graph implementation i distinguish between
  buffer kinds only based on block_size and amount of
  channels.
  this is because the sample rate arguably doesn't matter as
  it is a higher level concept than is neccessary on the
  level of the audio graph
  - distinguish between buffers that can be played back and
    others in the audio_buffer crate
