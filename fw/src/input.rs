use crate::sequencer::SequencerState;
use rtt_target::rprintln;

#[derive(Clone, Copy, Debug)]
pub enum Button {
    Step(u8),    // 0-15
    Track(u8),   // 0-7
    Pattern(u8), // 0-15
    Play,
    Stop,
}

pub fn handle_button_press(button: Button, seq: &mut SequencerState) {
    match button {
        Button::Step(n) => {
            let track = seq.selected_track as usize;
            let pattern = &mut seq.patterns[seq.visible_pattern as usize];
            let step = &mut pattern.tracks[track].steps[n as usize];
            step.active = !step.active;
            if step.active && step.pitch == 0 {
                step.pitch = 60; // Default to C4
            }
            rprintln!("Step {} on track {}: {}", n, track, step.active);
        }
        Button::Track(n) => {
            seq.selected_track = n;
            rprintln!("Selected track {}", n);
        }
        Button::Pattern(n) => {
            seq.visible_pattern = n;
            rprintln!("Selected pattern {}", n);
        }
        Button::Play => {
            rprintln!("Play");
            // TODO: implement play/pause toggle
        }
        Button::Stop => {
            rprintln!("Stop");
            // TODO: implement stop
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

        b'a' => Some(Button::Track(0)),
        b's' => Some(Button::Track(1)),
        b'd' => Some(Button::Track(2)),
        b'f' => Some(Button::Track(3)),
        b'g' => Some(Button::Track(4)),
        b'h' => Some(Button::Track(5)),
        b'j' => Some(Button::Track(6)),
        b'k' => Some(Button::Track(7)),

        // Transport
        b' ' => Some(Button::Play),  // Space = play/pause
        b'\n' => Some(Button::Stop), // Enter = stop
        _ => None,
    }
}
