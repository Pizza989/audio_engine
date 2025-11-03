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
- add multi track support to the timeline
  - figure out how to design an audio_graph that fits this
