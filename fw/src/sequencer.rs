pub struct SequencerState {
    max_steps: u8,
    current_step: u8,
}

impl Default for SequencerState {
    fn default() -> Self {
        Self {
            max_steps: 16,
            current_step: 1,
        }
    }
}

impl SequencerState {
    pub fn new() -> Self {
        Self::default()
    }
}

