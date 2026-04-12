use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::thread::sleep;
use std::time::Duration;

use std::sync::{Arc, Mutex};

use std::io;

use super::VirtualKeyboard;
use crate::keycodes::{Key, KeyEvent, KeyEvents, KeypressType};

impl VirtualKeyboard {
    pub fn new() -> Result<Self, io::Error> {
        Ok(VirtualKeyboard {
            modifiers: (false, false, false),
        })
    }

    pub fn write_code(&mut self, event: KeyEvent) -> Result<(), io::Error> {
        //println!("mac event: {:?}", event);
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();

        if event.code() == Key::new("shift").code {
            // shift
            match event {
                KeyEvent::Press(keypress) => {
                    self.modifiers.0 = true;
                }
                KeyEvent::Release(keypress) => {
                    self.modifiers.0 = false;
                }
                _ => {}
            };
            //println!("shifting: {:?}", self.modifiers);
            // return Ok(());
        }
        if event.code() == Key::new("leftalt").code {
            // alt
            match event {
                KeyEvent::Press(keypress) => {
                    self.modifiers.1 = true;
                }
                KeyEvent::Release(keypress) => {
                    self.modifiers.1 = false;
                }
                _ => {}
            };
            //println!("alting: {:?}", self.modifiers);
            // return Ok(());
        }
        if event.code() == Key::new("leftctrl").code {
            // ctrl
            match event {
                KeyEvent::Press(keypress) => {
                    self.modifiers.2 = true;
                }
                KeyEvent::Release(keypress) => {
                    self.modifiers.2 = false;
                }
                _ => {}
            };
            //println!("ctrling: {:?}", self.modifiers);
            // return Ok(());
        }

        let is_down = matches!(event, KeyEvent::Press(_));

        let cgevent;

        match event {
            KeyEvent::Press(keypress) => {
                cgevent = CGEvent::new_keyboard_event(source, keypress.code as u16, true).unwrap();
            }
            KeyEvent::Release(keypress) => {
                cgevent = CGEvent::new_keyboard_event(source, keypress.code as u16, false).unwrap();
            }
        }

        // Set the event flags if needed (e.g., for modifier keys)
        let mut flags = CGEventFlags::empty();
        if self.modifiers.0 == true {
            flags.insert(CGEventFlags::CGEventFlagShift);
        }
        if self.modifiers.1 == true {
            flags.insert(CGEventFlags::CGEventFlagAlternate);
        }
        if self.modifiers.2 == true {
            flags.insert(CGEventFlags::CGEventFlagControl);
        }
        cgevent.set_flags(flags);
        cgevent.post(CGEventTapLocation::HID);

        Ok(())
    }

    pub fn write_many(&mut self, keypresses: KeyEvents) {
        keypresses.iter().for_each(|event| {
            self.write_code(event.clone());
        })
    }
}
