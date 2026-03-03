use midi_event::Note;

use super::Key;
use crate::{output_methods::InputMethod, keycodes::{KeyEvent, KeyEvents}};

const PIANO_ROOMS_KEYS: [&str; 12] = [
    "kp0", "kp1", "kp2", "kp3", "kp4", "kp5", "kp6", "kp7", "kp8", "kp9", "kpminus", "kpplus"
];

pub struct Inner;

impl InputMethod for Inner {
    fn get_name(&self) -> String {
        "Piano Rooms".to_owned()
    }

    fn press_note(&mut self, note: Note, velocity: u8) -> KeyEvents {
        println!("[PianoRooms]: Playing note {:?} at velocity {}", note, velocity);

        // original code:
        //      Array = ['num0', 'numpad1', 'numpad2', 'numpad3', 'numpad4', 'numpad5', 'numpad6', 'numpad7', 'numpad8', 'numpad9', 'subtract', 'add']
        //      print(str(msg.note) + " " + str(msg.velocity))
        //      ToSend = [math.floor(msg.note/12),math.floor(msg.note%12),math.floor(msg.velocity/12),math.floor(msg.velocity%12)]

        let mut events: KeyEvents = vec![
            KeyEvent::Press(Key::new("kpasterisk")),
            KeyEvent::Release(Key::new("kpasterisk"))
        ];

        let to_send: Vec<u8> = vec![note as u8 / 12, note as u8 % 12, velocity / 12, velocity % 12];
        to_send.iter().for_each(|num| {
            let key = PIANO_ROOMS_KEYS.get(*num as usize).expect("Invalid Piano Rooms key");
            events.push(KeyEvent::Press(Key::new(key)));
            events.push(KeyEvent::Release(Key::new(key)));
        });

        events
    }

    fn release_note(&mut self, note: Note) -> KeyEvents {
        println!("[PianoRooms]: Releasing note: {:?}", note);

        let mut events: KeyEvents = vec![
            KeyEvent::Press(Key::new("kpasterisk")),
            KeyEvent::Release(Key::new("kpasterisk"))
        ];

        let to_send: Vec<u8> = vec![note as u8 / 12, note as u8 % 12, 0, 0];
        to_send.iter().for_each(|num| {
            let key = PIANO_ROOMS_KEYS.get(*num as usize).expect("Invalid Piano Rooms key");
            events.push(KeyEvent::Press(Key::new(key)));
            events.push(KeyEvent::Release(Key::new(key)));
        });

        events
    }

    fn reset(&mut self, data: &str) {}

    fn process_sustain(&mut self, value: u8) -> KeyEvents {
        println!("[PianoRooms]: Processing sustain: {}", value);

        let mut events: KeyEvents = vec![
            KeyEvent::Press(Key::new("kpasterisk")),
            KeyEvent::Release(Key::new("kpasterisk"))
        ];

        let control = 143;
        let to_send: Vec<u8> = vec![control / 12, control % 12, value / 12, value % 12];
        to_send.iter().for_each(|num| {
            let key = PIANO_ROOMS_KEYS.get(*num as usize).expect("Invalid Piano Rooms key");
            events.push(KeyEvent::Press(Key::new(key)));
            events.push(KeyEvent::Release(Key::new(key)));
        });

        events
    }

    fn process_sostenuto(&mut self, value: u8) -> KeyEvents {
        println!("[PianoRooms]: Processing sostenuto: {}", value);
        // Piano Rooms uses a custom protocol; sostenuto not in original spec
        vec![]
    }
}