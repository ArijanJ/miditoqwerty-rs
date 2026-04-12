use evdev::*;
use std::io;

use super::VirtualKeyboard;

use crate::keycodes::{KeyEvent, KeyEvents, KeypressType, UniversalKeyCode};

impl VirtualKeyboard {
    pub fn new() -> Result<Self, io::Error> {
        let keys = evdev::AttributeSet::from_iter((0..560).map(evdev::Key));

        let device = uinput::VirtualDeviceBuilder::new()?
            .name("miditoqwerty")
            .input_id(evdev::InputId::new(evdev::BusType::BUS_I8042, 1, 1, 1))
            .with_keys(&keys);

        let mut device = device.expect("Failed to build device").build()?;
        let devnode = device
            .enumerate_dev_nodes_blocking()
            .unwrap()
            .next() // Expect only one. Using fold or calling next again blocks indefinitely
            .ok_or_else(|| {
                io::Error::new(std::io::ErrorKind::NotFound, "devnode is not found")
            })??;
        println!("Created device {:#?}", devnode);

        Ok(VirtualKeyboard {
            device,
            raw_buf: vec![],
        })
    }

    pub fn write_code(&mut self, event: KeyEvent) -> Result<(), io::Error> {
        match event {
            KeyEvent::Press(keypress) => {
                self.device.emit(&[InputEvent::new(
                    EventType::KEY,
                    keypress.code,
                    KeypressType::Press as i32,
                )]);
            }
            KeyEvent::Release(keypress) => {
                self.device.emit(&[InputEvent::new(
                    EventType::KEY,
                    keypress.code,
                    KeypressType::Release as i32,
                )]);
            }
        }

        Ok(())
    }

    pub fn write_many(&mut self, keypresses: KeyEvents) {
        keypresses.iter().for_each(|event| {
            self.write_code(event.clone());
        })
    }
}
