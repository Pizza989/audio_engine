# Links
https://github.com/MeadowlarkDAW/meadowlark-core-types/blob/main/src/time/superclock_time.rs

# Processing Loop
```rust
  for block in timeline.iter_blocks(block_size) {
    let events = timeline.get_events(block);

    for event in events {
      let track = event.track;
      track.node.send_event(event);
    }

    run_graph.process_block();
    let output_buffer = run_graph.graph_output_buffers();
    // write output_buffer to audio backends buffer
  }
```

# TODO
(1) REASON: to make it convenient to hand slices of audio clips from
    the cache to the audio graph
    TASK: rework the dsp-chain crate to use audio's buffers this
    includes:
    * implementing support for variable amounts of inputs and out-
      puts per node
    * obviously rewriting the audio processing logic in the graph
(2) REASON: The timeline currently only supports single events with-
    out context. To make it possible to place audio clips in the
    timeline some form of context needs to exist.
    TASK: make the timeline store Clips instead of events. these will
    have a start and an end and the timeline can therefore correctly
    generate events to schedule playing any part of the clip
