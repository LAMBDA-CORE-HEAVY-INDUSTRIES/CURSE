use core::fmt::Write;
use core::sync::atomic::Ordering;

use embedded_hal::digital::OutputPin;

use crate::sequencer::{BPM, CURRENT_STEP, SequencerState};
use crate::utils::FmtBuf;

// const GRID_COLOR: u32 = 0x3F9834;
// const GRID_COLOR: u32 = 0xD79128;
const GRID_COLOR: u32 = 0x134213;
// const GRID_COLOR: u32 = 0x05b669;
const NUM_STEPS: u16 = 16;
const ROW_HEIGHT: u16 = 64;
const GRID_LEFT: u16 = 56;
const GRID_RIGHT: u16 = 1000;
const GRID_TOP: u16 = 40;
const CELL_WIDTH: u16 = (GRID_RIGHT - GRID_LEFT) / NUM_STEPS;
const TRACK_LABELS: [&str; 8] = ["00", "01", "02", "03", "04", "05", "06", "07"];
const NUM_TRACKS: usize = TRACK_LABELS.len();

pub fn render<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
) {
    // display.draw_rectangle(0, 0, 1023, 598 , GRID_COLOR, false);
    let mut buf = [0u8; 32];
    let mut fmt = FmtBuf::new(&mut buf);
    write!(fmt, "BPM:{}", BPM.load(Ordering::Relaxed)).unwrap();
    let _ = display.write_text_scaled(fmt.as_str(), GRID_RIGHT - 160, 6, None, 0xf07826, 2, 2);
    for (i, label) in TRACK_LABELS.iter().enumerate() {
        let y1 = GRID_TOP + (i as u16) * ROW_HEIGHT;
        let y2 = y1 + ROW_HEIGHT;
        let _ = display.draw_rectangle(GRID_LEFT, y1, GRID_RIGHT, y2, GRID_COLOR, false);
        let _ = display.write_text(label, GRID_LEFT - 40, y1 + 25, None, GRID_COLOR);
    }
    for n in 1..NUM_STEPS {
        let x = GRID_LEFT + (n * CELL_WIDTH);
        let _ = display.draw_line(x, GRID_TOP, x, ROW_HEIGHT * 9 - 24, GRID_COLOR);
    }

    for n in 0..NUM_STEPS {
        render_steps(display, sequencer_state, n as u8, false);
    }
}

pub fn render_steps<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
    step_index: u8,
    active: bool,
) {
    let color = if step_index % 4 == 0 { 0x000000 } else { 0x121212 };
    let bg_color = if active { 0x444444 } else { color };
    for (i, _label) in TRACK_LABELS.iter().enumerate() {
        let y = GRID_TOP + (i as u16) * ROW_HEIGHT;
        let text_y = y + (ROW_HEIGHT / 2) - 6;
        let x = GRID_LEFT + (step_index as u16 * CELL_WIDTH);
        let text_x = x + (CELL_WIDTH / 2) - 6;
        let _ = display.draw_rectangle(x + 1, y + 1, x + CELL_WIDTH - 2, y + ROW_HEIGHT - 1,  bg_color, true);
        let pattern = &sequencer_state.patterns[sequencer_state.visible_pattern as usize];
        let step = pattern.tracks[i].steps[step_index as usize];
        let _ = display.write_text(step.as_str(), text_x, text_y, None, if step.active && step.pitch != 0 {0x949494 } else { 0x333333 });
    }
}

pub fn render_selected_step<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
    step_index: u8,
) {
    let bg_color = 0xFFFFFF;
    let i = sequencer_state.selected_track as usize;
    let y = GRID_TOP + (i as u16) * ROW_HEIGHT;
    let text_y = y + (ROW_HEIGHT / 2) - 6;
    let x = GRID_LEFT + (step_index as u16 * CELL_WIDTH);
    let text_x = x + (CELL_WIDTH / 2) - 6;
    let _ = display.draw_rectangle(x + 1, y + 1, x + CELL_WIDTH - 2, y + ROW_HEIGHT - 1,  bg_color, true);
    let pattern = &sequencer_state.patterns[sequencer_state.visible_pattern as usize];
    let step = pattern.tracks[i].steps[step_index as usize];
    let _ = display.write_text(step.as_str(), text_x, text_y, None, 0x00000);
}
