use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use stm32f4xx_hal::pac::{self, TIM3};
use stm32f4xx_hal::timer::CounterHz;
use stm32f4xx_hal::{interrupt, prelude::_fugit_RateExtU32};

pub static BPM: AtomicU32 = AtomicU32::new(120);
pub static PPQN: AtomicU32 = AtomicU32::new(24);
pub static CURRENT_STEP: AtomicU8 = AtomicU8::new(0);
pub static PREVIOUS_STEP: AtomicU8 = AtomicU8::new(0);
pub static TICK: AtomicU32 = AtomicU32::new(0);
pub static STEP_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Default)]
pub struct Step {
    pub active: bool,
    pub pitch: u8,
}

impl Step {
    pub fn as_str(&self) -> &'static str {
        if !self.active { return "--"; }
        match self.pitch {
            60 => "C4",
            61 => "C#4",
            62 => "D4",
            _ => "??",
        }
    }
}

pub struct SequencerState {
    pub max_steps: u8,
    pub steps: [[Step; 16]; 8],
}

impl Default for SequencerState {
    fn default() -> Self {
        Self {
            max_steps: 16,
            steps: [[Step::default(); 16]; 8],
        }
    }
}

impl SequencerState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[interrupt]
fn TIM3() {
    unsafe {
        // Clear reason for the generated interrupt request
        (*pac::TIM3::ptr()).sr().modify(|_, w| w.uif().clear_bit());
    }
    let tick = TICK.fetch_add(1, Ordering::Relaxed);
    let ticks_per_step = PPQN.load(Ordering::Relaxed) / 4;
    if tick > ticks_per_step {
        TICK.store(0, Ordering::Relaxed);
        let max_steps = 16;
        let step = CURRENT_STEP.load(Ordering::Relaxed);
        PREVIOUS_STEP.store(step, Ordering::Relaxed);
        CURRENT_STEP.store((step + 1) % max_steps, Ordering::Relaxed);
        STEP_FLAG.store(true, Ordering::Release);
    }
}

pub fn set_bpm(timer: &mut CounterHz<TIM3>, bpm: u32) {
    BPM.store(bpm, Ordering::Relaxed);
    let tick_freq = (bpm * PPQN.load(Ordering::Relaxed)) / 60;
    timer.start(tick_freq.Hz()).unwrap();
}

pub fn set_step(sequencer_state: &mut SequencerState, track_index: u8, step_index: u8, pitch: u8){
    sequencer_state.steps[track_index as usize][step_index as usize].pitch = pitch;
}
