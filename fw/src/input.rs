use core::sync::atomic::Ordering;

use crate::sequencer::{SequencerState, PLAYING, select_step, set_step};
use rtt_target::rprintln;

#[derive(Clone, Copy, Debug)]
pub enum Button {
    Step(u8),    // 0-15
    Track(u8),   // 0-7
    Pattern(u8), // 0-15
    Note(u8),
    OctaveUp,
    OctaveDown,
    Play,
    Stop,
}

pub fn handle_button_press(button: Button, sequencer_state: &mut SequencerState) {
    match button {
        Button::Step(n) => {
            let track = sequencer_state.selected_track as usize;
            let pattern = &mut sequencer_state.patterns[sequencer_state.visible_pattern as usize];
            let step = &mut pattern.tracks[track].steps[n as usize];
            step.active = !step.active;
            if step.active && step.pitch == 0 {
                step.pitch = 60; // Default to C4
            }
            rprintln!("Step {} on track {}: {}", n, track, step.active);
            select_step(sequencer_state, n);
        }
        Button::Track(n) => {
            sequencer_state.selected_track = n;
            rprintln!("Selected track {}", n);
        }
        Button::Pattern(n) => {
            sequencer_state.visible_pattern = n;
            rprintln!("Selected pattern {}", n);
        }
        Button::Play => {
            let was_playing = PLAYING.fetch_xor(true, Ordering::Relaxed);
            rprintln!("{}", if was_playing { "Pause" } else { "Play" });
        }
        Button::Stop => {
            PLAYING.store(false, Ordering::Relaxed);
            rprintln!("Stop");
        }
        Button::Note(n) => {
            rprintln!("note: {}", n);
            let track = sequencer_state.selected_track;
            let selected_step = match sequencer_state.selected_step {
                Some(s) => s,
                None => return,
            };
            set_step(sequencer_state, track, selected_step, n);
        }
        Button::OctaveUp => {
            rprintln!("octave up");
        }
        Button::OctaveDown => {
            rprintln!("octave down");
        }
    }
}

#[cfg(feature = "keyboard-input")]
pub fn key_to_button(key: u8) -> Option<Button> {
    match key {
        b'1' => Some(Button::Step(0)),
        b'2' => Some(Button::Step(1)),
        b'3' => Some(Button::Step(2)),
        b'4' => Some(Button::Step(3)),
        b'5' => Some(Button::Step(4)),
        b'6' => Some(Button::Step(5)),
        b'7' => Some(Button::Step(6)),
        b'8' => Some(Button::Step(7)),
        b'9' => Some(Button::Step(8)),
        b'0' => Some(Button::Step(9)),
        b'q' => Some(Button::Step(10)),
        b'w' => Some(Button::Step(11)),
        b'e' => Some(Button::Step(12)),
        b'r' => Some(Button::Step(13)),
        b't' => Some(Button::Step(14)),
        b'y' => Some(Button::Step(15)),

        // Shift+1-0 for US kb layouts
        b'!' => Some(Button::Track(0)),
        b'@' => Some(Button::Track(1)),
        b'#' => Some(Button::Track(2)),
        b'$' => Some(Button::Track(3)),
        b'%' => Some(Button::Track(4)),
        b'^' => Some(Button::Track(5)),
        b'&' => Some(Button::Track(6)),
        b'*' => Some(Button::Track(7)),

        b'z' => Some(Button::Note(60)),
        b's' => Some(Button::Note(61)),
        b'x' => Some(Button::Note(62)),
        b'd' => Some(Button::Note(63)),
        b'c' => Some(Button::Note(64)),
        b'v' => Some(Button::Note(65)),
        b'g' => Some(Button::Note(66)),
        b'b' => Some(Button::Note(67)),
        b'h' => Some(Button::Note(68)),
        b'n' => Some(Button::Note(69)),
        b'j' => Some(Button::Note(70)),
        b'm' => Some(Button::Note(71)),

        b'+' => Some(Button::OctaveUp),
        b'-' => Some(Button::OctaveDown),

        b' ' => Some(Button::Play),
        b'x' => Some(Button::Stop),
        _ => None,
    }
}
