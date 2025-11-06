pub struct PinMatrix {
    rows: usize,
    cols: usize,
    data: Vec<bool>,
}

impl PinMatrix {
    pub fn new(input_channels: usize, output_channels: usize) -> Self {
        Self {
            rows: output_channels,
            cols: input_channels,
            data: vec![false; output_channels * input_channels],
        }
    }

    pub fn diagonal(input_channels: usize, output_channels: usize) -> Self {
        let mut result = vec![false; input_channels * output_channels];

        let diagonal_length = input_channels.min(output_channels);

        for i in 0..diagonal_length {
            result[i * input_channels + i] = true;
        }

        Self {
            rows: output_channels,
            cols: input_channels,
            data: result,
        }
    }

    pub fn full(input_channels: usize, output_channels: usize) -> Self {
        Self {
            rows: output_channels,
            cols: input_channels,
            data: vec![true; output_channels * input_channels],
        }
    }

    pub fn get(&self, input_channel: usize, output_channel: usize) -> bool {
        self.data[output_channel * self.cols + input_channel]
    }

    pub fn set(&mut self, input_channel: usize, output_channel: usize, val: bool) {
        self.data[output_channel * self.cols + input_channel] = val;
    }

    pub fn input_channels(&self) -> usize {
        self.cols
    }

    pub fn output_channels(&self) -> usize {
        self.rows
    }

    /// Converts to a list of input to output connections
    pub fn channel_connections(&self) -> Vec<(usize, usize)> {
        let mut pins = vec![];
        for (index, pin) in self.data.iter().enumerate() {
            if *pin {
                pins.push((index % self.cols, index / self.cols));
            }
        }
        pins
    }
}
