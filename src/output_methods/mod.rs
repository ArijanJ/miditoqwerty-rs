#![allow(unused_variables)]
#![allow(dead_code)]

use midi_event::Note;

use crate::keycodes::{KeyEvents, Key};

mod generic;
mod pv;
mod piano_rooms;

pub trait InputMethod {
    fn get_name(&self) -> String;
    fn press_note(&mut self, note: Note, velocity: u8) -> KeyEvents;
    fn release_note(&mut self, note: Note) -> KeyEvents;
    fn reset(&mut self, data: &str);
    fn process_sustain(&mut self, value: u8) -> KeyEvents;
}


pub mod unified {
    pub use super::generic::Inner as generic_inner;
    pub use super::pv::Inner as pv_inner;
    pub use super::piano_rooms::Inner as piano_rooms_inner;
}