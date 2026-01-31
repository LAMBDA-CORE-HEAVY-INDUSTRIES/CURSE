use core::fmt::Write;
use core::sync::atomic::Ordering;

use embedded_hal::digital::OutputPin;

use crate::sequencer::{SequencerState, BPM};
use crate::utils::FmtBuf;

const GRID_COLOR: u32 = 0x3F9834;
const NUM_STEPS: u16 = 16;
const ROW_HEIGHT: u16 = 60;
const GRID_LEFT: u16 = 80;
const GRID_RIGHT: u16 = 1000;
const GRID_TOP: u16 = 60;
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
    write!(fmt, "BPM: {}", BPM.load(Ordering::Relaxed)).unwrap();
    let _ = display.write_text(fmt.as_str(), GRID_RIGHT - 100, 5, None, 0x949494);
    for (i, label) in TRACK_LABELS.iter().enumerate() {
        let y1 = GRID_TOP + (i as u16) * ROW_HEIGHT;
        let y2 = y1 + ROW_HEIGHT;
        let _ = display.draw_rectangle(GRID_LEFT, y1, GRID_RIGHT, y2, GRID_COLOR, false);
        let _ = display.write_text(label, GRID_LEFT - 40, y1 + 25, None, GRID_COLOR);
    }
    for n in 1..NUM_STEPS {
        let x = GRID_LEFT + (n * CELL_WIDTH);
        let _ = display.draw_line(x, GRID_TOP, x, ROW_HEIGHT * 9, GRID_COLOR);
    }

    for (i, _label) in TRACK_LABELS.iter().enumerate() {
        let mut y = GRID_TOP + (i as u16) * ROW_HEIGHT;
        let text_y = y + (ROW_HEIGHT / 2) - 6;
        for n in 0..NUM_STEPS {
            let mut x = GRID_LEFT + (n * CELL_WIDTH);
            if n % 4 == 0 {
                display.draw_rectangle(x + 1, y + 1, x + CELL_WIDTH - 2, y + ROW_HEIGHT - 1,  0x111111, true);
            }
            x = x + (CELL_WIDTH / 2) - 6;
            if n % 2 == 0 {
                let _ = display.write_text("c2", x, text_y, None, 0x949494);
            } else {
                let _ = display.write_text("---", x - 6, text_y, None, 0x949494);
            }
        }
    }
}


pub fn render_steps<I: lt7683::LT7683Interface, RESET: OutputPin>(
    display: &mut lt7683::LT7683<I, RESET>,
    sequencer_state: &SequencerState,
    step_index: u8,
    active: bool,
) {
    let bg_color = if active { 0x000000 } else { 0x222222 };
    for (i, _label) in TRACK_LABELS.iter().enumerate() {
        let y = GRID_TOP + (i as u16) * ROW_HEIGHT;
        let text_y = y + (ROW_HEIGHT / 2) - 6;
        let mut x = GRID_LEFT + (step_index as u16 * CELL_WIDTH);
        let text_x = x + (CELL_WIDTH / 2) - 6;
        display.draw_rectangle(x + 1, y + 1, x + CELL_WIDTH - 2, y + ROW_HEIGHT - 1,  bg_color, true);
        display.write_text(sequencer_state.steps[i][step_index as usize].as_str(), text_x, text_y, None, 0x949494);
    }
}
