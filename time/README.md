Abstractions over time taken from meadowlark-core-types that are designed for accurate
timekeeping in a DAW. SuperclockTime has been renamed to SampleTime. All conversion
functions that require other types from the crate besides MusicalTime and
SuperclockTime have been commented out. **Should they be uncommented at some point, then
the ones that are not lossless should be postfixed with the word lossy.**
