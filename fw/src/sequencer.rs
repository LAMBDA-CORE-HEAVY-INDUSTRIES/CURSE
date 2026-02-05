use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use rtt_target::rprintln;
use stm32f4xx_hal::pac::{self, TIM3};
use stm32f4xx_hal::timer::CounterHz;
use stm32f4xx_hal::{interrupt, prelude::_fugit_RateExtU32};

use crate::utils::iter_bits;

pub static BPM: AtomicU32 = AtomicU32::new(120);
pub static PPQN: AtomicU32 = AtomicU32::new(24);
pub static NEXT_STEP: AtomicU8 = AtomicU8::new(0);
pub static CURRENT_STEP: AtomicU8 = AtomicU8::new(0);
pub static TICK: AtomicU32 = AtomicU32::new(0);
pub static STEP_FLAG: AtomicBool = AtomicBool::new(false);
pub static EDIT_FLAG: AtomicBool = AtomicBool::new(false);
pub static PLAYING: AtomicBool = AtomicBool::new(false);

pub const MAX_TRACKS: usize = 8;
pub const MAX_STEPS: usize = 16;
pub const MAX_PATTERNS: usize = 16;
pub const MAX_SONG_LENGTH: usize = 64;

pub static mut SEQ: SequencerState = SequencerState::new();

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
        }
        EDIT_FLAG.store(true, Ordering::Release);
    }

    pub fn select_only_track(&mut self, track: u8) {
        self.selected_tracks = 1 << track;
        EDIT_FLAG.store(true, Ordering::Release);
    }

    pub fn selected_tracks_iter(&self) -> impl Iterator<Item = u8> {
        iter_bits(self.selected_tracks)
    }

    pub fn get_all_tracks(&self) -> u8 {
        0xFF
    }
}

#[interrupt]
fn TIM3() {
    unsafe {
        // Clear reason for the generated interrupt request
        (*pac::TIM3::ptr()).sr().modify(|_, w| w.uif().clear_bit());
    }
    if !PLAYING.load(Ordering::Relaxed) {
        return;
    }
    let tick = TICK.fetch_add(1, Ordering::Relaxed);
    let ticks_per_step = PPQN.load(Ordering::Relaxed) / 4;
    if tick > ticks_per_step {
        TICK.store(0, Ordering::Relaxed);
        let step = NEXT_STEP.load(Ordering::Relaxed);
        CURRENT_STEP.store(step, Ordering::Relaxed);
        STEP_FLAG.store(true, Ordering::Release);
        // NOTE: We might want to use shift register if running out of GPIO.
        // TODO: Iterate all tracks and set gpio low/high if active. Now just checking for track 3.
        // TODO: Gate length
        unsafe {
            let gpioa = &(*pac::GPIOA::ptr());
            let sequencer_state = { &mut *(&raw mut SEQ) };
            let pattern = sequencer_state.get_playing_pattern();
            // TODO: For now we just use the first track length. We can utilize different length
            // tracks in the future for polymetric things.
            let length = pattern.tracks[0].length;
            NEXT_STEP.store((step + 1) % length, Ordering::Relaxed);
            if pattern.tracks[2].steps[step as usize].active {
                rprintln!("step {} is active", step);
                gpioa.bsrr().write(|w| w.bs10().set_bit());
            } else {
                rprintln!("step {} is not active", step);
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

pub fn select_step(seq: &mut SequencerState, step_index: u8){
    seq.prev_selected_step = seq.selected_step;
    seq.selected_step = Some(step_index);
    EDIT_FLAG.store(true, Ordering::Release);
}

pub fn set_step(sequencer_state: &mut SequencerState, tracks: u8, step_index: u8, pitch: u8){
    let pattern = &mut sequencer_state.patterns[sequencer_state.visible_pattern as usize];
    for track_index in iter_bits(tracks) {
        pattern.tracks[track_index as usize].steps[step_index as usize].pitch = pitch;
        // TODO: toggle active
        pattern.tracks[track_index as usize].steps[step_index as usize].active = true;
    }
    EDIT_FLAG.store(true, Ordering::Release);
}
