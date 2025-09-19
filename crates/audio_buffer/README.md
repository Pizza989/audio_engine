# TODO
- rework the core/io module
- if multiple mutable borrows to different sections of a buffer should ever be
  required. one could use a recursive implementation by putting an associated
  type on the Buffer that requires Buffer. the details would however be
  managed by the implementor as the Buffer traits is agnostic to memory layout
