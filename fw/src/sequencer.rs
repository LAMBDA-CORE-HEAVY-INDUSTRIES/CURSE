use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use stm32f4xx_hal::pac::{self, TIM3};
use stm32f4xx_hal::{interrupt, rcc::Clocks};

use crate::utils::iter_bits_u8;

pub static BPM: AtomicU32 = AtomicU32::new(120);
pub static PPQN: AtomicU32 = AtomicU32::new(24);
pub static NEXT_STEP: AtomicU8 = AtomicU8::new(0);
pub static CURRENT_STEP: AtomicU8 = AtomicU8::new(0);
pub static STEP_FLAG: AtomicBool = AtomicBool::new(false);
pub static PLAYING: AtomicBool = AtomicBool::new(false);

const TIMER_HZ: u32 = 1_000_000;
const MAX_SEGMENT_US: u32 = 0xFFFF;

pub const MAX_TRACKS: usize = 8;
pub const MAX_STEPS: usize = 16;
pub const MAX_PATTERNS: usize = 16;
pub const MAX_SONG_LENGTH: usize = 64;

pub static mut SEQ: SequencerState = SequencerState::new();

pub const DIRTY_STEP_SELECTION: u8 = 0x01;
pub const DIRTY_TRACK_SELECTION: u8 = 0x02;
pub const DIRTY_NOTE_DATA: u8 = 0x04;
pub const DIRTY_BPM: u8 = 0x08;
pub const DIRTY_PATTERN: u8 = 0x10;
pub const DIRTY_RT_CACHE: u8 = 0x20;
static DIRTY: AtomicU8 = AtomicU8::new(0);

pub fn mark_dirty(flags: u8) {
    DIRTY.fetch_or(flags, Ordering::Release);
}

pub fn take_dirty() -> u8 {
    DIRTY.swap(0, Ordering::Acquire)
}

pub struct RtCache {
    pub gate_masks: [u16; MAX_TRACKS],
    pub pitches: [[u8; MAX_STEPS]; MAX_TRACKS],
    pub lengths: [u8; MAX_TRACKS],
    pub gate_lengths: [u8; MAX_TRACKS],
}

impl RtCache {
    pub const fn new() -> Self {
        Self {
            gate_masks: [0; MAX_TRACKS],
            pitches: [[0; MAX_STEPS]; MAX_TRACKS],
            lengths: [0; MAX_TRACKS],
            gate_lengths: [0; MAX_TRACKS],
        }
    }
}

static mut RT_CACHE: [RtCache; 2] = [RtCache::new(), RtCache::new()];
static ACTIVE_CACHE: AtomicU8 = AtomicU8::new(0);

struct StepInterval {
    base_us: u32,
    rem: u32,
    denom: u32,
    acc: u32,
}

impl StepInterval {
    const fn new() -> Self {
        Self {
            base_us: 1,
            rem: 0,
            denom: 1,
            acc: 0,
        }
    }
}

static mut STEP_INTERVAL: StepInterval = StepInterval::new();
static mut REMAINING_US: u32 = 0;
static mut LAST_CCR1: u16 = 0;
static mut HAS_LAST_CCR1: bool = false;

#[derive(Clone, Copy, Default, Debug)]
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
            _ => "--",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Track {
    pub steps: [Step; MAX_STEPS],
    pub length: u8,
}

impl Track {
    pub const fn new() -> Self {
        Self {
            steps: [Step::new(); MAX_STEPS],
            length: MAX_STEPS as u8,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Song {
    pub entries: [u8; MAX_SONG_LENGTH],
    pub length: u8,
}

impl Song {
    pub const fn new() -> Self {
        Self {
            entries: [0; MAX_SONG_LENGTH],
            length: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pattern {
    pub tracks: [Track; MAX_TRACKS],
}

impl Pattern {
    pub const fn new() -> Self {
        Self {
            tracks: [Track::new(); MAX_TRACKS],
        }
    }

    pub fn set_length(&mut self, len: u8) {
        // NOTE: for now, we are setting all tracks the same length
        for track in &mut self.tracks {
            track.length = len;
        }
        mark_dirty(DIRTY_PATTERN | DIRTY_RT_CACHE);
    }
}

pub struct SequencerState {
    pub max_steps: u8,
    pub patterns: [Pattern; MAX_PATTERNS],
    pub song: Song,

    pub play_mode: PlayMode,
    pub song_position: u8,
    pub step_position: u8,

    pub visible_pattern: u8,
    pub playing_pattern: u8,
    pub selected_tracks: u8,

    // NOTE: By storing the selected step like this, we are basically restricting ourselves to
    // being able to edit only the steps that are currently visible (visible_pattern). So if there
    // are multiple patterns, once the current pattern changes, we need to either set the
    // selected_step to None, or keep it as is, which means that user would be editing step from
    // the next pattern. tldr: for now, no way to edit step of a pattern that is not visible
    // anymore.
    pub selected_step: Option<u8>,
    pub prev_selected_step: Option<u8>,
}

#[derive(Clone, Copy)]
pub enum PlayMode {
    Pattern,
    Song,
}

impl SequencerState {
    pub const fn new() -> Self {
        Self {
            max_steps: MAX_STEPS as u8,
            patterns: [Pattern::new(); MAX_PATTERNS],
            song: Song::new(),
            play_mode: PlayMode::Pattern,
            song_position: 0,
            step_position: 0,
            visible_pattern: 0,
            playing_pattern: 0,
            selected_tracks: 1,
            selected_step: None,
            prev_selected_step: None,
        }
    }

    #[inline]
    pub fn get_playing_pattern(&self) -> &Pattern {
        let pattern_index = match self.play_mode {
            PlayMode::Pattern => self.playing_pattern,
            PlayMode::Song => self.song.entries[self.song_position as usize],
        };
        &self.patterns[pattern_index as usize]
    }

    pub fn is_track_selected(&self, track: u8) -> bool {
        self.selected_tracks & (1 << track) != 0
    }

    pub fn toggle_track(&mut self, track: u8) {
        let selected_tracks = self.selected_tracks ^ (1 << track);
        if selected_tracks != 0 {
            self.selected_tracks = selected_tracks;
            mark_dirty(DIRTY_TRACK_SELECTION);
        }
    }

    pub fn select_only_track(&mut self, track: u8) {
        self.selected_tracks = 1 << track;
        mark_dirty(DIRTY_TRACK_SELECTION);
    }

    pub fn selected_tracks_iter(&self) -> impl Iterator<Item = u8> {
        iter_bits_u8(self.selected_tracks)
    }

    pub fn get_all_tracks(&self) -> u8 {
        0xFF
    }
}

#[interrupt]
fn TIM3() {
    unsafe {
        // Clear interrupt flags (compare + update)
        let tim3 = &*pac::TIM3::ptr();
        tim3.sr()
            .modify(|_, w| w.cc1if().clear_bit().uif().clear_bit());
    }

    if unsafe { REMAINING_US == 0 } {
        if PLAYING.load(Ordering::Relaxed) {
            let step = NEXT_STEP.load(Ordering::Relaxed);
            CURRENT_STEP.store(step, Ordering::Relaxed);
            STEP_FLAG.store(true, Ordering::Release);
            // NOTE: We might want to use shift register if running out of GPIO.
            // TODO: Iterate all tracks and set gpio low/high if active. Now just checking for track 3.
            // TODO: Gate length
            unsafe {
                let gpioa = &(*pac::GPIOA::ptr());
                let cache_index = ACTIVE_CACHE.load(Ordering::Acquire);
                let cache = &RT_CACHE[cache_index as usize];
                // TODO: For now we just use the first track length. We can utilize different length
                // tracks in the future for polymetric things.
                let length = cache.lengths[0].min(MAX_STEPS as u8);
                if length != 0 {
                    NEXT_STEP.store((step + 1) % length, Ordering::Relaxed);
                    let gate_mask = cache.gate_masks[2];
                    if gate_mask & (1u16 << step) != 0 {
                        // rprintln!("step {} is active", step);
                        gpioa.bsrr().write(|w| w.bs10().set_bit());
                    } else {
                        // rprintln!("step {} is not active", step);
                        gpioa.bsrr().write(|w| w.br10().set_bit());
                    }
                }
            }
        }

        unsafe {
            REMAINING_US = next_interval_us();
        }
    }

    unsafe {
        schedule_next_segment();
    }
}

pub fn rebuild_rt_cache(sequencer_state: &SequencerState) {
    let active = ACTIVE_CACHE.load(Ordering::Acquire);
    let inactive = active ^ 1;
    let cache = unsafe { &mut RT_CACHE[inactive as usize] };
    let pattern = sequencer_state.get_playing_pattern();
    for track_index in 0..MAX_TRACKS {
        let track = &pattern.tracks[track_index];
        cache.lengths[track_index] = track.length;
        // Placeholder for future per-track gate length control.
        cache.gate_lengths[track_index] = 1;

        let mut mask: u16 = 0;
        for step_index in 0..MAX_STEPS {
            let step = track.steps[step_index];
            cache.pitches[track_index][step_index] = step.pitch;
            if step.active {
                mask |= 1u16 << step_index;
            }
        }
        cache.gate_masks[track_index] = mask;
    }
    ACTIVE_CACHE.store(inactive, Ordering::Release);
}

fn pulses_per_step_from_ppqn(ppqn: u32) -> Option<u32> {
    match ppqn {
        4 => Some(1),
        24 => Some(6),
        _ => None,
    }
}

pub fn init_step_timer(tim3: TIM3, clocks: &Clocks) {
    unsafe {
        let rcc = &*pac::RCC::ptr();
        rcc.apb1enr().modify(|_, w| w.tim3en().set_bit());
        rcc.apb1rstr().modify(|_, w| w.tim3rst().set_bit());
        rcc.apb1rstr().modify(|_, w| w.tim3rst().clear_bit());
    }

    tim3.cr1().modify(|_, w| w.cen().clear_bit());
    let timclk = clocks.timclk1().raw();
    let prescaler = (timclk / TIMER_HZ).saturating_sub(1);
    tim3.psc().write(|w| unsafe { w.psc().bits(prescaler as u16) });
    tim3.arr().write(|w| unsafe { w.arr().bits(0xFFFF) });
    tim3.cnt().write(|w| unsafe { w.cnt().bits(0) });
    tim3.egr().write(|w| w.ug().set_bit());
    tim3.sr().modify(|_, w| w.cc1if().clear_bit().uif().clear_bit());
}

pub fn set_bpm(bpm: u32) {
    BPM.store(bpm, Ordering::Relaxed);
    mark_dirty(DIRTY_BPM);
    let ppqn = PPQN.load(Ordering::Relaxed);
    let pulses_per_step = pulses_per_step_from_ppqn(ppqn).unwrap_or(1);
    let denom = bpm.saturating_mul(ppqn).max(1);
    let numer = 60_000_000u64 * pulses_per_step as u64;
    let base_us = (numer / denom as u64) as u32;
    let rem = (numer % denom as u64) as u32;

    cortex_m::interrupt::free(|_| unsafe {
        STEP_INTERVAL.base_us = base_us.max(1);
        STEP_INTERVAL.rem = rem;
        STEP_INTERVAL.denom = denom;
        STEP_INTERVAL.acc = 0;
        HAS_LAST_CCR1 = false;
        REMAINING_US = next_interval_us();
        schedule_next_segment();
        let tim3 = &*pac::TIM3::ptr();
        tim3.dier().modify(|_, w| w.cc1ie().set_bit().uie().clear_bit());
        tim3.cr1().modify(|_, w| w.cen().set_bit());
    });
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn next_interval_us() -> u32 {
    let mut acc = STEP_INTERVAL.acc;
    let rem = STEP_INTERVAL.rem;
    let denom = STEP_INTERVAL.denom;
    let mut extra = 0;
    if rem != 0 && denom != 0 {
        acc = acc.wrapping_add(rem);
        if acc >= denom {
            acc = acc.wrapping_sub(denom);
            extra = 1;
        }
        STEP_INTERVAL.acc = acc;
    }
    STEP_INTERVAL.base_us.saturating_add(extra).max(1)
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn schedule_next_segment() {
    let mut remaining = REMAINING_US.max(1);
    let segment = if remaining > MAX_SEGMENT_US {
        MAX_SEGMENT_US as u16
    } else {
        remaining as u16
    };
    remaining = remaining.saturating_sub(segment as u32);
    REMAINING_US = remaining;

    let tim3 = &*pac::TIM3::ptr();
    let cnt = tim3.cnt().read().cnt().bits();
    let mut last = LAST_CCR1;
    if !HAS_LAST_CCR1 {
        last = cnt;
        HAS_LAST_CCR1 = true;
    }
    let seg = segment.max(1) as u32;
    let elapsed = cnt.wrapping_sub(last) as u32;
    let mut next = last.wrapping_add(segment);
    if elapsed >= seg {
        let missed = (elapsed / seg).saturating_add(1);
        let add = (seg.saturating_mul(missed)) as u16;
        next = last.wrapping_add(add);
    }
    LAST_CCR1 = next;
    tim3.ccr1().write(|w| unsafe { w.ccr().bits(next) });
}

pub fn select_step(seq: &mut SequencerState, step_index: u8) {
    seq.prev_selected_step = seq.selected_step;
    seq.selected_step = Some(step_index);
    mark_dirty(DIRTY_STEP_SELECTION);
}

pub fn set_step(sequencer_state: &mut SequencerState, tracks: u8, step_index: u8, pitch: u8) {
    let pattern = &mut sequencer_state.patterns[sequencer_state.visible_pattern as usize];
    for track_index in iter_bits_u8(tracks) {
        pattern.tracks[track_index as usize].steps[step_index as usize].pitch = pitch;
        // TODO: toggle active
        pattern.tracks[track_index as usize].steps[step_index as usize].active = true;
    }
    mark_dirty(DIRTY_NOTE_DATA | DIRTY_RT_CACHE);
}
