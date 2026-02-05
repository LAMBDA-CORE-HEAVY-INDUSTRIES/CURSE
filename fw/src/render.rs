use core::fmt::Write;
use core::sync::atomic::Ordering;

use embedded_hal::digital::OutputPin;

use crate::sequencer::{BPM, SequencerState};
use crate::utils::{FmtBuf, iter_bits};

// const GRID_COLOR: u32 = 0x3F9834;
// const GRID_COLOR: u32 = 0xD79128;
const COLOR_GRID_FG: u32 = 0x134213;
const COLOR_CELL_BG: u32 = 0x121212;
const COLOR_CELL_ACTIVE_BG: u32 = 0x444444;
const COLOR_CELL_SECONDARY_BG: u32 = 0x000000;
const COLOR_CELL_SELECTED_BG: u32 = 0x05b669;
const COLOR_TRACK_LABEL_FG: u32 = COLOR_GRID_FG;
const COLOR_TRACK_LABEL_ACTIVE_FG: u32 = 0xF07826;
// const GRID_COLOR: u32 = 0x05b669;
const NUM_STEPS: u16 = 16;
const ROW_HEIGHT: u16 = 64;
const GRID_LEFT: u16 = 56;
const GRID_RIGHT: u16 = 1000;
const GRID_TOP: u16 = 40;
const CELL_WIDTH: u16 = (GRID_RIGHT - GRID_LEFT) / NUM_STEPS;
const TRACK_LABELS: [&str; 8] = ["00", "01", "02", "03", "04", "05", "06", "07"];

pub fn render<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
) {
    // display.draw_rectangle(0, 0, 1023, 598 , GRID_COLOR, false);
    let mut buf = [0u8; 32];
    let mut fmt = FmtBuf::new(&mut buf);
    write!(fmt, "BPM:{}", BPM.load(Ordering::Relaxed)).unwrap();
    let _ = display.write_text_scaled(fmt.as_str(), GRID_RIGHT - 160, 6, None, 0xf07826, 2, 2);
    for track_index in iter_bits(sequencer_state.get_all_tracks()) {
        let y1 = GRID_TOP + (track_index as u16) * ROW_HEIGHT;
        let y2 = y1 + ROW_HEIGHT;
        let _ = display.draw_rectangle(GRID_LEFT, y1, GRID_RIGHT, y2, COLOR_GRID_FG, false);
        render_track_label(display, track_index, false);
    }
    for n in 1..NUM_STEPS {
        let x = GRID_LEFT + (n * CELL_WIDTH);
        let _ = display.draw_line(x, GRID_TOP, x, ROW_HEIGHT * 9 - 24, COLOR_GRID_FG);
    }

    for n in 0..NUM_STEPS {
        render_column(display, sequencer_state, n as u8, CellHighlight::None);
    }
}

pub fn render_track_label<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    track_index: u8,
    selected: bool,
) {
    let color_fg = if selected { COLOR_TRACK_LABEL_ACTIVE_FG } else { COLOR_TRACK_LABEL_FG };
    let y = GRID_TOP + (track_index as u16) * ROW_HEIGHT + 25;
    let _ = display.write_text(TRACK_LABELS[track_index as usize], GRID_LEFT - 40, y, None, color_fg);
}


#[derive(Clone, Copy, PartialEq)]
pub enum CellHighlight {
    None,
    Playing,
    Selected,
}

pub fn render_cell<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
    track_index: u8,
    step_index: u8,
    highlight: CellHighlight,
) {
    let base_bg = if step_index % 4 == 0 { COLOR_CELL_SECONDARY_BG } else { COLOR_CELL_BG };
    let bg_color = match highlight {
        CellHighlight::None => base_bg,
        CellHighlight::Playing => COLOR_CELL_ACTIVE_BG,
        CellHighlight::Selected => COLOR_CELL_SELECTED_BG,
    };

    let y = GRID_TOP + (track_index as u16) * ROW_HEIGHT;
    let x = GRID_LEFT + (step_index as u16 * CELL_WIDTH);
    let text_y = y + (ROW_HEIGHT / 2) - 6;
    let text_x = x + (CELL_WIDTH / 2) - 6;
    let _ = display.draw_rectangle(x + 1, y + 1, x + CELL_WIDTH - 2, y + ROW_HEIGHT - 1, bg_color, true);

    let pattern = &sequencer_state.patterns[sequencer_state.visible_pattern as usize];
    let step = pattern.tracks[track_index as usize].steps[step_index as usize];
    let text_color = match highlight {
        CellHighlight::Selected => 0x000000,
        _ => if step.active && step.pitch != 0 { 0x949494 } else { 0x333333 },
    };
    let _ = display.write_text(step.as_str(), text_x, text_y, None, text_color);
}

pub fn render_column<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
    step_index: u8,
    highlight: CellHighlight,
) {
    for track_index in iter_bits(sequencer_state.get_all_tracks()) {
        render_cell(display, sequencer_state, track_index, step_index, highlight);
    }
}

pub fn render_cells<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
    step_index: u8,
    tracks: u8,
    highlight: CellHighlight,
) {
    for track_index in iter_bits(tracks) {
        render_cell(display, sequencer_state, track_index, step_index, highlight);
    }
}
