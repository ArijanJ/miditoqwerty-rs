pub const REGULAR_VP_NOTES: &[u8] = "1!2@34$5%6^78*9(0qQwWeErtTyYuiIoOpPasSdDfgGhHjJklLzZxcCvVbBnm".as_bytes();

use midi_event::Note;

use super::InputMethod;
use crate::keycodes::{ Key, KeyEvent, KeyEvents};

fn str_for_note(note: Note) -> Option<String> {
    let note_value = note as usize;

    if note >= Note::C2 && note <= Note::C7 {
        Some((*REGULAR_VP_NOTES.get(note as usize - Note::C2 as usize).unwrap() as char).to_string())
    } else {
        None
    }
}

pub struct Inner{
    pressed_chars: [u8; 127], // OS key codes [idx] -> times pressed [u8]
    space_down: bool,
}

impl Inner {
    pub fn new() -> Self {
        Inner {pressed_chars: [0; 127], space_down: false}
    }
}

impl InputMethod for Inner {
    fn get_name(&self) -> String {
        "Generic".to_owned()
    }

    fn press_note(&mut self, note: Note, velocity: u8) -> KeyEvents {
        let mut events: KeyEvents = Vec::new();
        println!("[Generic]: Playing note {} ({:?}) at velocity {}", note as u32, note, velocity);

        let key_string = str_for_note(note);

        if key_string.is_none() {
            println!("[Generic]: Impossible to press note {:?} with generic", note);
            return vec![];
        } let key_string = key_string.unwrap();

        events.push(KeyEvent::Release(Key::new(&key_string))); // release in case it was already held

        let keypress = Key::new(&key_string);

        if keypress.shifted { events.push(KeyEvent::Press(Key::new("shift"))) };
        events.push(KeyEvent::Press(keypress));
        self.pressed_chars[keypress.code as usize] += 1;
        if keypress.shifted { events.push(KeyEvent::Release(Key::new("shift"))) };

        //println!("{:?}", self.pressed_chars);

        events
    }

    fn release_note(&mut self, note: Note) -> KeyEvents {
        let events: KeyEvents = vec![];
        println!("[Generic]: Releasing note: {} ({:?})", note as u32, note);

        let key_string = str_for_note(note);

        if key_string.is_none() {
            println!("[Generic]: Impossible to press note {:?} with generic", note);
            return vec![];
        } let key_string = key_string.unwrap();

        let keypress = Key::new(&key_string);
        //println!("{:?}", keypress.code);

        let presses = self.pressed_chars[keypress.code as usize];
        if presses != 0 {
            self.pressed_chars[keypress.code as usize] -= 1;
        }

        if presses > 1 {
            // Don't actually release
            //println!("{:?}", self.pressed_chars);
            return vec![];
        }

        vec![KeyEvent::Release(keypress)]
    }

    fn reset(&mut self, data: &str) {
        self.pressed_chars = [0; 127];
        self.space_down = false;
    }

    fn process_sustain(&mut self, value: u8) -> KeyEvents {
        println!("[Generic]: Processing sustain: {}", value);

        if value >= 64 && !self.space_down {
            self.space_down = true;
            vec![KeyEvent::Press(Key::new("space"))]
        } else if value < 64 && self.space_down {
            self.space_down = false;
            vec![KeyEvent::Release(Key::new("space"))]
        } else {
            vec![]
        }
    }
}