use core::ptr::addr_of_mut;
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

pub static mut SEQ: SequencerState = SequencerState::new();

#[derive(Clone, Copy, Default)]
pub struct Step {
    pub active: bool,
    pub pitch: u8,
}

impl Step {
    pub const fn new() -> Self {
        Self { active: false, pitch: 0 }
    }

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

impl SequencerState {
    pub const fn new() -> Self {
        Self {
            max_steps: 16,
            steps: [[Step::new(); 16]; 8],
        }
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
        // NOTE: We might want to use shift register if running out of GPIO.
        // TODO: Iterate all tracks and set gpio low/high if active. Now just checking for track 3.
        // TODO: Gate length
        unsafe {
            let gpioa = &(*pac::GPIOA::ptr());
            let sequencer_state = { &mut *addr_of_mut!(SEQ) };
            if sequencer_state.steps[2][step as usize].active {
                defmt::trace!("step {:?} is active", step);
                gpioa.bsrr().write(|w| w.bs10().set_bit());
            } else {
                defmt::trace!("step {:?} is not active", step);
                gpioa.bsrr().write(|w| w.br10().set_bit());
            }
        }
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
