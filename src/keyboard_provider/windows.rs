extern crate winapi;
use std::mem::size_of;
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VK_RETURN,
};

use std::io;

use super::VirtualKeyboard;
use crate::keycodes::{KeyEvent, KeyEvents, KeypressType};

fn send_scancode(scancode: u16, status: KeypressType) {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe { std::mem::zeroed() },
    };

    // Set up input
    unsafe {
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: 0,
            wScan: scancode,
            dwFlags: KEYEVENTF_SCANCODE
                | if matches!(status, KeypressType::Release) {
                    KEYEVENTF_KEYUP
                } else {
                    0
                },
            time: 0,
            dwExtraInfo: 0,
        };
    }

    unsafe {
        SendInput(1, &mut input, size_of::<INPUT>() as i32);
    }
}

impl VirtualKeyboard {
    pub fn new() -> Result<Self, io::Error> {
        Ok(VirtualKeyboard {}) // Windows is simple, just one API call needed to send input
    }

    pub fn write_code(&mut self, event: KeyEvent) -> Result<(), io::Error> {
        match event {
            KeyEvent::Press(keypress) => {
                send_scancode(keypress.code.try_into().unwrap(), KeypressType::Press);
            }
            KeyEvent::Release(keypress) => {
                send_scancode(keypress.code.try_into().unwrap(), KeypressType::Release);
            }
        };
        Ok(())
    }

    pub fn write_many(&mut self, events: KeyEvents) {
        events.iter().for_each(|event| {
            self.write_code(event.clone());
        })
    }
}
