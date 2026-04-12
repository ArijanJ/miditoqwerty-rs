#![allow(dead_code)]
#![allow(unused)]

use std::path::Display;

#[cfg(target_os = "linux")]
use evdev::{uinput, InputEvent};

use std::sync::{Arc, Mutex};

#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSource;

struct KeyboardProvider {}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub struct VirtualKeyboard {
    device: uinput::VirtualDevice,
    raw_buf: Vec<InputEvent>,
}

#[cfg(target_os = "windows")]
pub struct VirtualKeyboard {}
// everything can be done at runtime ^, no need to create new stuff

#[cfg(target_os = "macos")]
pub struct VirtualKeyboard {
    pub modifiers: (bool, bool, bool), // 1=shift, 2=alt. TODO enum(?) later
                                       // source: Arc<Mutex<CGEventSource>>
} // whatever, just create new ones

impl std::fmt::Debug for VirtualKeyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "yes i am a keyboard")
    }
}

pub fn create_virtual_keyboard() -> VirtualKeyboard {
    VirtualKeyboard::new().unwrap()
}
