use stm32f4xx_hal::prelude::_fugit_RateExtU32;
use stm32f4xx_hal::pac::TIM3;
use stm32f4xx_hal::timer::CounterHz;
use core::sync::atomic::{AtomicU32, Ordering};

pub struct SequencerState {
    pub max_steps: u8,
    pub current_step: u8,
    pub bpm: AtomicU32,
    pub ppqn: u32,
}

impl Default for SequencerState {
    fn default() -> Self {
        Self {
            max_steps: 16,
            current_step: 1,
            bpm: AtomicU32::new(120),
            ppqn: 24,
        }
    }
}

impl SequencerState {
    pub fn new() -> Self {
        Self::default()
    }
}


pub fn set_bpm(sequencer_state: &mut SequencerState, timer: &mut CounterHz<TIM3>, bpm: u32) {
    sequencer_state.bpm.store(bpm, Ordering::Relaxed);
    let tick_freq = bpm * sequencer_state.ppqn;
    timer.start(tick_freq.Hz()).unwrap();
}
