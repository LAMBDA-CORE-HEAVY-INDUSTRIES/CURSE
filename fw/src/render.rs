use core::fmt::Write;
use core::sync::atomic::Ordering;

use embedded_hal::digital::OutputPin;

use crate::sequencer::{BPM, SequencerState};
use crate::utils::{FmtBuf, iter_bits_u8};

const COLOR_BG: u32 = 0x000000;
const COLOR_FRAME: u32 = 0x222222;
const COLOR_GRID_FG: u32 = 0x134213;
const COLOR_SIDEBAR_BG: u32 = 0x000000;
// const COLOR_GRID_FG: u32 = 0x222222;
const COLOR_CELL_BG: u32 = 0x121212;
const COLOR_CELL_SECONDARY_BG: u32 = 0x000000;
// const COLOR_CELL_SELECTED_BG: u32 = 0x05b669;
const COLOR_CELL_SELECTED_BG: u32 = 0x3F9834;
const COLOR_PLAYHEAD_FG: u32 = 0xF07826;
const COLOR_TRACK_LABEL_FG: u32 = COLOR_GRID_FG;
const COLOR_TRACK_LABEL_ACTIVE_FG: u32 = 0xF07826;

const COLOR_ACCENT_BG: u32 = 0x134213;

const SCREEN_W: u16 = 1024;
const SCREEN_H: u16 = 600;
const SCREEN_RIGHT: u16 = SCREEN_W - 1;
const SCREEN_BOTTOM: u16 = SCREEN_H - 2;
const HEADER_H: u16 = 0;
const BOTTOM_H: u16 = 48;
const SIDEBAR_W: u16 = 56;
const GRID_PADDING_X: u16 = 4;
const GRID_PADDING_Y: u16 = 16;
const NUM_STEPS: u16 = 16;
const ROW_HEIGHT: u16 = 64;
const GRID_LEFT: u16 = SIDEBAR_W + GRID_PADDING_X;
const GRID_TOP: u16 = HEADER_H + GRID_PADDING_Y;
const GRID_WIDTH: u16 = (SCREEN_RIGHT - GRID_PADDING_X) - GRID_LEFT;
const CELL_WIDTH: u16 = GRID_WIDTH / NUM_STEPS;
const GRID_RIGHT: u16 = GRID_LEFT + (CELL_WIDTH * NUM_STEPS);
const TRACK_LABELS: [&str; 8] = ["00", "01", "02", "03", "04", "05", "06", "07"];

const BOTTOM_LABEL_CONTAINER_HEIGHT: u16 = 32;
const BOTTOM_GAP: u16 = 4;

const PATTERN_AREA_X: u16 = 12;
const PATTERN_AREA_Y: u16 = SCREEN_H - BOTTOM_H;
const PATTERN_AREA_W: u16 = 28;
const PATTERN_TEXT_H: u16 = 16;
const PATTERN_TEXT_X: u16 = PATTERN_AREA_X + 8;
const PATTERN_TEXT_Y: u16 = PATTERN_AREA_Y + (BOTTOM_H / 2) - (PATTERN_TEXT_H / 2);

const BPM_AREA_X: u16 = PATTERN_AREA_X + PATTERN_AREA_W + BOTTOM_GAP + 14;
const BPM_AREA_Y: u16 = PATTERN_AREA_Y;
const BPM_AREA_W: u16 = 28;
const BPM_TEXT_H: u16 = 16;
const BPM_TEXT_X: u16 = BPM_AREA_X + 10;
const BPM_TEXT_Y: u16 = BPM_AREA_Y + (BOTTOM_H / 2) - (BPM_TEXT_H / 2);
const LABEL_X: u16 = 22;

pub fn render<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
) {
    render_frame(display);
    render_pattern_indicator(display, sequencer_state);
    render_bpm(display);
    for track_index in iter_bits_u8(sequencer_state.get_all_tracks()) {
        let y1 = GRID_TOP + (track_index as u16) * ROW_HEIGHT;
        let y2 = y1 + ROW_HEIGHT;
        let _ = display.draw_rectangle(GRID_LEFT, y1, GRID_RIGHT, y2, COLOR_GRID_FG, false);
        render_track_label(display, track_index, sequencer_state.is_track_selected(track_index));
    }
    for n in 1..NUM_STEPS {
        let x = GRID_LEFT + (n * CELL_WIDTH);
        let y_bottom = GRID_TOP + (ROW_HEIGHT * 8);
        let _ = display.draw_line(x, GRID_TOP, x, y_bottom, COLOR_GRID_FG);
    }

    for n in 0..NUM_STEPS {
        render_column(display, sequencer_state, n as u8, CellHighlight::None);
    }
}

pub fn render_frame<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
) {
    let _ = display.bte_solid_fill(0, 0, SCREEN_W, SCREEN_H, COLOR_BG);
    let _ = display.bte_solid_fill(0, HEADER_H, SIDEBAR_W, SCREEN_H - HEADER_H, COLOR_SIDEBAR_BG);
    let _ = display.bte_solid_fill(0, SCREEN_H - BOTTOM_H, SCREEN_W, BOTTOM_H, COLOR_SIDEBAR_BG);
    let _ = display.draw_line(0, SCREEN_H - BOTTOM_H, SCREEN_RIGHT, SCREEN_H - BOTTOM_H, COLOR_FRAME);
    let _ = display.draw_rectangle(0, 0, SCREEN_RIGHT, SCREEN_BOTTOM, COLOR_FRAME, false);
}

pub fn render_pattern_indicator<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
) {
    let mut buf = [0u8; 8];
    let mut fmt = FmtBuf::new(&mut buf);
    write!(fmt, "{:02}", sequencer_state.visible_pattern).unwrap();
    let bottom_y1 = SCREEN_H - BOTTOM_H + 6;
    let _ = display.bte_solid_fill(
        PATTERN_AREA_X,
        bottom_y1,
        PATTERN_AREA_X + PATTERN_AREA_W - 8,
        BOTTOM_LABEL_CONTAINER_HEIGHT,
        COLOR_ACCENT_BG,
    );
    let _ = display.write_text(fmt.as_str(), PATTERN_TEXT_X, PATTERN_TEXT_Y, None, COLOR_SIDEBAR_BG);
}

pub fn render_bpm<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
) {
    let mut buf = [0u8; 32];
    let mut fmt = FmtBuf::new(&mut buf);
    let bottom_y1 = SCREEN_H - BOTTOM_H + 6;
    let _ = display.bte_solid_fill(
        BPM_AREA_X,
        bottom_y1,
        BPM_AREA_X + BPM_AREA_W - 8,
        BOTTOM_LABEL_CONTAINER_HEIGHT,
        COLOR_ACCENT_BG,
    );
    write!(fmt, "BPM:{}", BPM.load(Ordering::Relaxed)).unwrap();
    let _ = display.write_text(fmt.as_str(), BPM_TEXT_X, BPM_TEXT_Y, None, COLOR_SIDEBAR_BG);
}

pub fn render_track_label<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    track_index: u8,
    selected: bool,
) {
    let color_fg = if selected { COLOR_TRACK_LABEL_ACTIVE_FG } else { COLOR_TRACK_LABEL_FG };
    let y = GRID_TOP + (track_index as u16) * ROW_HEIGHT;
    let text_y = y + 24;
    let _ = display.write_text(TRACK_LABELS[track_index as usize], LABEL_X, text_y, None, color_fg);
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
        CellHighlight::Playing => base_bg,
        CellHighlight::Selected => COLOR_CELL_SELECTED_BG,
    };

    let y = GRID_TOP + (track_index as u16) * ROW_HEIGHT;
    let x = GRID_LEFT + (step_index as u16 * CELL_WIDTH);
    let text_y = y + (ROW_HEIGHT / 2) - 6;
    let text_x = x + (CELL_WIDTH / 2) - 6;
    let _ = display.bte_solid_fill(x + 1, y + 1, CELL_WIDTH - 2, ROW_HEIGHT - 1, bg_color);

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
    for track_index in iter_bits_u8(sequencer_state.get_all_tracks()) {
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
    for track_index in iter_bits_u8(tracks) {
        render_cell(display, sequencer_state, track_index, step_index, highlight);
    }
}

pub fn render_playhead_marker<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    step_index: u8,
    is_playing: bool,
) {
    let color = if is_playing { COLOR_PLAYHEAD_FG } else { COLOR_GRID_FG };
    let x1 = GRID_LEFT + (step_index as u16 * CELL_WIDTH) + 1;
    let x2 = x1 + CELL_WIDTH - 2;
    let y_top = GRID_TOP;
    let y_bottom = GRID_TOP + (ROW_HEIGHT * 8);
    let _ = display.draw_line(x1, y_top, x2, y_top, color);
    let _ = display.draw_line(x1, y_bottom, x2, y_bottom, color);
}
