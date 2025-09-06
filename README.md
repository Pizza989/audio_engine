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
(1) pull in the audio crate and extend it so that it can interop with symphonia.
(2) then continue pulling in the symphonium source code and convert it to use
    the types from the audio crate.

(1) this envolves improving the conversion between foreign buffer types
      * this might mean reworking the current model of wrapping them
        using the wrap module
      * this might alternatively mean adding additional implementations
        to the wrapped Buffers

(2) this means reworking the src/audio/decode.rs file so the decode functions
    generate the correct buffer types


(1') create an AudioBuffer type that
      * can be loaded from a file
      * can be shared somehow
