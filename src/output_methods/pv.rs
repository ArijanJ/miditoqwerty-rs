pub const REGULAR_VP_NOTES: &[u8] = "1!2@34$5%6^78*9(0qQwWeErtTyYuiIoOpPasSdDfgGhHjJklLzZxcCvVbBnm".as_bytes();
pub const LOW_VP_NOTES: &[u8] = "trewq0987654321".as_bytes();
pub const HIGH_VP_NOTES: &[u8] = "yuiopasdfghj".as_bytes();

use midi_event::Note;

use crate::keycodes::KeyEvent;

use super::{KeyEvents, InputMethod, Key};

pub fn string_for_velocity(velocity: u8) -> String {
    const VELOCITY_KEYS: &[u8] = "1234567890qwertyuiopasdfghjklzxc".as_bytes();
    const VELOCITY_LIST: [u8; 32] = [
        4, 8, 12, 16,
        20, 24, 28, 32,
        36, 40, 44, 48,
        52, 56, 60, 64,
        68, 72, 76, 80,
        84, 88, 92, 96,
        100, 104, 108, 112,
        116, 120, 124, 127,
    ];

    let (index, closest) = VELOCITY_LIST
        .iter()
        .enumerate()
        .min_by_key(|&(_, &v)| (velocity as i32 - v as i32).abs())
        .unwrap_or((0, &VELOCITY_LIST[15])); // Default to index 15

    // println!("velocity: {}", velocity);
    // println!("index: {}", index);
    (*VELOCITY_KEYS.get(index).expect("Velocity index issue") as char).to_string()
}

pub fn events_for_velocity(velocity: u8) -> KeyEvents {
    let mut events = vec![];

    events.push(KeyEvent::Press(Key::new("leftalt")));

    let velocity_keypress = string_for_velocity(velocity);

    // release all previous sames
    events.push(KeyEvent::Release(Key::new(&velocity_keypress)));

    events.push(KeyEvent::Press(Key::new(&velocity_keypress)));
    events.push(KeyEvent::Release(Key::new(&velocity_keypress)));

    events.push(KeyEvent::Release(Key::new("leftalt")));

    events
}

fn str_for_note(note: Note) -> Option<String> {
    let note_value = note as usize;

    if note < Note::C2 {
        Some((*LOW_VP_NOTES.get((note as i8 - Note::B1 as i8).abs() as usize).unwrap() as char).to_string())
    }
    else if note > Note::C7 {
        Some((*HIGH_VP_NOTES.get((note as i8 - Note::Cs7 as i8).abs() as usize).unwrap() as char).to_string())
    }
    else {
        Some((*REGULAR_VP_NOTES.get(note as usize - Note::C2 as usize).unwrap() as char).to_string())
    }
}

pub struct Inner {
    pressed_chars: [u8; 127], // OS key codes [idx] -> times pressed [u8]
    space_down: bool,
    velocity: bool
}

impl Inner {
    pub fn new() -> Self {
        Inner {pressed_chars: [0; 127], space_down: false, velocity: true}
    }
}


impl InputMethod for Inner {
    fn get_name(&self) -> String {
        "Piano Visualizations".to_owned()
    }

    fn press_note(&mut self, note: Note, velocity: u8) -> KeyEvents {
        let mut events: KeyEvents = Vec::new();
        println!("[PV]: Playing note {} ({:?}) at velocity {}", note as u32, note, velocity);

        if self.velocity {
            events.append(&mut events_for_velocity(velocity));
        }

        let is_88_key = note < Note::C2 || note > Note::C7;
        if is_88_key { events.push(KeyEvent::Press(Key::new("leftctrl"))) };

        let keystring = str_for_note(note).unwrap();
        let keypress = Key::new(&keystring); // meta / for info

        // Release just to make sure that we can actually play it again
        events.push(KeyEvent::Release(Key::new(&keystring)));

        if keypress.shifted { events.push(KeyEvent::Press(Key::new("shift"))) };
        events.push(KeyEvent::Press(Key::new(&keystring)));
        self.pressed_chars[keypress.code as usize] += 1;
        if keypress.shifted { events.push(KeyEvent::Release(Key::new("shift"))) };

        if is_88_key { events.push(KeyEvent::Release(Key::new("leftctrl"))) };

        events
    }

    fn release_note(&mut self, note: Note) -> KeyEvents {
        let events: KeyEvents = vec![];
        println!("[PV]: Releasing note: {} ({:?})", note as u32, note);

        let key_string = str_for_note(note);

        if key_string.is_none() {
            println!("[Generic]: Impossible to press note {:?} with PV", note);
            return vec![];
        } let key_string = key_string.unwrap();

        let keypress = Key::new(&key_string);

        let presses = self.pressed_chars[keypress.code as usize];
        if presses != 0 {
            self.pressed_chars[keypress.code as usize] -= 1;
        }

        if presses > 1 {
            // Don't actually release
            //println!("{:?}", self.pressed_chars);
            return vec![];
        }

        //println!("kprs: {keypress}");
        vec![KeyEvent::Release(keypress)]
    }

    fn reset(&mut self, data: &str) {
        self.pressed_chars = [0; 127];
        self.space_down = false;
        self.velocity = !data.starts_with("velocity-off");
    }

    fn process_sustain(&mut self, value: u8) -> KeyEvents {
        println!("[PV]: Processing sustain: {}", value);

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