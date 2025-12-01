use audio_buffer::{AudioBuffer, CopyOptions, dasp};

use crate::mix_graph::processor::{AudioTransformer, ProcessingContext};

pub struct TransformerChain<Sample>
where
    Sample: dasp::Sample,
{
    transformers: Vec<Box<dyn AudioTransformer<Sample>>>,
    back_buffer: AudioBuffer<Sample>,
}

impl<Sample> TransformerChain<Sample>
where
    Sample: dasp::Sample,
{
    pub fn new(num_channels: usize, num_frames: usize) -> Self {
        Self {
            transformers: Vec::new(),
            back_buffer: AudioBuffer::with_shape(num_channels, num_frames),
        }
    }

    pub fn push_transformer(&mut self, transformer: Box<dyn AudioTransformer<Sample>>) {
        self.transformers.push(transformer);
    }

    pub fn insert_transformer(
        &mut self,
        index: usize,
        transformer: Box<dyn AudioTransformer<Sample>>,
    ) {
        self.transformers.insert(index, transformer);
    }

    pub fn remove_transformer(
        &mut self,
        index: usize,
    ) -> Box<dyn AudioTransformer<Sample> + 'static> {
        self.transformers.remove(index)
    }

    // TODO: use unsafe to copy in place
    // This would improve performance because copying to a
    // back-buffer wouldn't be necessary and transformation
    // could truly happen in-place
    pub fn process_in_place(
        &mut self,
        buffer: &mut AudioBuffer<Sample>,
        context: ProcessingContext,
    ) {
        for transformer in self.transformers.iter_mut() {
            self.back_buffer
                .copy_from(&buffer, CopyOptions::default())
                .expect("not in strict mode and no offsets given");

            transformer.transform_in_place(&self.back_buffer, buffer, context.clone());
        }
    }
}
