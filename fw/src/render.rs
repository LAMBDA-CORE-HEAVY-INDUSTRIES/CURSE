use embedded_hal::digital::OutputPin;

use crate::sequencer::SequencerState;

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
}

