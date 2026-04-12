#![allow(non_camel_case_types)]
#![allow(dead_code)]

use phf::phf_map;

pub static SHIFTS: phf::Map<&str, &str> = phf_map! {
    "!" => "1", "A" => "a", "K" => "k", "U" => "u",
    "@" => "2", "B" => "b", "L" => "l", "V" => "v",
    "#" => "3", "C" => "c", "M" => "m", "W" => "w",
    "$" => "4", "D" => "d", "N" => "n", "X" => "x",
    "%" => "5", "E" => "e", "O" => "o", "Y" => "y",
    "^" => "6", "F" => "f", "P" => "p", "Z" => "z",
    "&" => "7", "G" => "g", "Q" => "q",
    "*" => "8", "H" => "h", "R" => "r",
    "(" => "9", "I" => "i", "S" => "s",
    ")" => "0", "J" => "j", "T" => "t",
};
pub enum KeypressType {
    Release = 0,
    Press = 1,
    Repeat = 2,
}

pub type KeyEvents = Vec<KeyEvent>;

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Press(Key),
    Release(Key),
}
impl KeyEvent {
    pub fn code(&self) -> u16 {
        match &self {
            KeyEvent::Press(key) => key.code.clone(),
            KeyEvent::Release(key) => key.code.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub code: u16,
    pub shifted: bool,
}
impl Key {
    pub fn new(s: &str) -> Self {
        println!("Trying to get {s}");
        let unshifted = SHIFTS.get(s);

        let code = match unshifted {
            Some(key) => UniversalKeyCode::get(key),
            None => UniversalKeyCode::get(s),
        };

        match code {
            None => panic!("Invalid key-string supplied"),
            Some(code) => Key {
                code: code.try_into().unwrap(),
                shifted: unshifted.is_some(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniversalKeyCode {
    windows: u32,
    mac: u32,
}
impl UniversalKeyCode {
    #[cfg(not(target_os = "macos"))]
    pub fn get(s: &str) -> Option<u32> {
        KEYCODES.get(s).map(|key| key.windows)
    }
    #[cfg(target_os = "macos")]
    pub fn get(s: &str) -> Option<u32> {
        KEYCODES.get(s).map(|key| key.mac)
    }
}

pub static KEYCODES: phf::Map<&'static str, UniversalKeyCode> = phf_map! {
    // "ESC" => UniversalKeyCode { windows: 1, mac:  },
    "1" => UniversalKeyCode { windows: 2, mac: 0x12 },
    "2" => UniversalKeyCode { windows: 3, mac: 0x13 },
    "3" => UniversalKeyCode { windows: 4, mac: 0x14 },
    "4" => UniversalKeyCode { windows: 5, mac: 0x15 },
    "5" => UniversalKeyCode { windows: 6, mac: 0x17 },
    "6" => UniversalKeyCode { windows: 7, mac: 0x16 },
    "7" => UniversalKeyCode { windows: 8, mac: 0x1a },
    "8" => UniversalKeyCode { windows: 9, mac: 0x1c },
    "9" => UniversalKeyCode { windows: 10, mac: 0x19 },
    "0" => UniversalKeyCode { windows: 11, mac: 0x1d },
    // "MINUS" => UniversalKeyCode { windows: 12, mac:  },
    // "EQUAL" => UniversalKeyCode { windows: 13, mac:  },
    "backspace" => UniversalKeyCode { windows: 14, mac: 0x2a },
    // "TAB" => UniversalKeyCode { windows: 15, mac:  },
    "q" => UniversalKeyCode { windows: 16, mac: 0x0c },
    "w" => UniversalKeyCode { windows: 17, mac: 0x0d },
    "e" => UniversalKeyCode { windows: 18, mac: 0x0e },
    "r" => UniversalKeyCode { windows: 19, mac: 0x0f },
    "t" => UniversalKeyCode { windows: 20, mac: 0x11 },
    "y" => UniversalKeyCode { windows: 21, mac: 0x10 },
    "u" => UniversalKeyCode { windows: 22, mac: 0x20 },
    "i" => UniversalKeyCode { windows: 23, mac: 0x22 },
    "o" => UniversalKeyCode { windows: 24, mac: 0x1f },
    "p" => UniversalKeyCode { windows: 25, mac: 0x23 },
    "leftbrace" => UniversalKeyCode { windows: 26, mac: 0x21 },
    "rightbrace" => UniversalKeyCode { windows: 27, mac: 0x1e },
    // "enter" => UniversalKeyCode { windows: 28, mac:  },
    "leftctrl" => UniversalKeyCode { windows: 29, mac: 0x3b },
    "a" => UniversalKeyCode { windows: 30, mac: 0x00 },
    "s" => UniversalKeyCode { windows: 31, mac: 0x01 },
    "d" => UniversalKeyCode { windows: 32, mac: 0x02 },
    "f" => UniversalKeyCode { windows: 33, mac: 0x03 },
    "g" => UniversalKeyCode { windows: 34, mac: 0x05 },
    "h" => UniversalKeyCode { windows: 35, mac: 0x04 },
    "j" => UniversalKeyCode { windows: 36, mac: 0x26 },
    "k" => UniversalKeyCode { windows: 37, mac: 0x28 },
    "l" => UniversalKeyCode { windows: 38, mac: 0x25 },
    "semicolon" => UniversalKeyCode { windows: 39, mac: 0x29 },
    // "apostrophe" => UniversalKeyCode { windows: 40, mac:  },
    "grave" => UniversalKeyCode { windows: 41, mac: 0x32 },
    "shift" => UniversalKeyCode { windows: 42, mac: 0x38 },
    // "backslash" => UniversalKeyCode { windows: 43, mac:  },
    "z" => UniversalKeyCode { windows: 44, mac: 0x06 },
    "x" => UniversalKeyCode { windows: 45, mac: 0x07 },
    "c" => UniversalKeyCode { windows: 46, mac: 0x08 },
    "v" => UniversalKeyCode { windows: 47, mac: 0x09 },
    "b" => UniversalKeyCode { windows: 48, mac: 0x0b },
    "n" => UniversalKeyCode { windows: 49, mac: 0x2d },
    "m" => UniversalKeyCode { windows: 50, mac: 0x2e },
    // "COMMA" => UniversalKeyCode { windows: 51, mac:  },
    // "DOT" => UniversalKeyCode { windows: 52, mac:  },
    // "SLASH" => UniversalKeyCode { windows: 53, mac:  },
    // "RIGHTSHIFT" => UniversalKeyCode { windows: 54, mac:  },
    "kpasterisk" => UniversalKeyCode { windows: 55, mac: 0x43 },
    "leftalt" => UniversalKeyCode { windows: 56, mac: 0x3a },
    "space" => UniversalKeyCode { windows: 57, mac: 0x31 },
    // "CAPSLOCK" => UniversalKeyCode { windows: 58, mac:  },
    // "F1" => UniversalKeyCode { windows: 59, mac:  },
    // "F2" => UniversalKeyCode { windows: 60, mac:  },
    // "F3" => UniversalKeyCode { windows: 61, mac:  },
    // "F4" => UniversalKeyCode { windows: 62, mac:  },
    // "F5" => UniversalKeyCode { windows: 63, mac:  },
    // "F6" => UniversalKeyCode { windows: 64, mac:  },
    // "F7" => UniversalKeyCode { windows: 65, mac:  },
    // "F8" => UniversalKeyCode { windows: 66, mac:  },
    // "F9" => UniversalKeyCode { windows: 67, mac:  },
    // "F10" => UniversalKeyCode { windows: 68, mac:  },
    // "NUMLOCK" => UniversalKeyCode { windows: 69, mac:  },
    // "SCROLLLOCK" => UniversalKeyCode { windows: 70, mac:  },
    "kp7" => UniversalKeyCode { windows: 71, mac: 0x59 },
    "kp8" => UniversalKeyCode { windows: 72, mac: 0x5b },
    "kp9" => UniversalKeyCode { windows: 73, mac: 0x5c },
    "kpminus" => UniversalKeyCode { windows: 74, mac: 0x4e },
    "kp4" => UniversalKeyCode { windows: 75, mac: 0x56 },
    "kp5" => UniversalKeyCode { windows: 76, mac: 0x57 },
    "kp6" => UniversalKeyCode { windows: 77, mac: 0x58 },
    "kpplus" => UniversalKeyCode { windows: 78, mac: 0x45 },
    "kp1" => UniversalKeyCode { windows: 79, mac: 0x53 },
    "kp2" => UniversalKeyCode { windows: 80, mac: 0x54 },
    "kp3" => UniversalKeyCode { windows: 81, mac: 0x55 },
    "kp0" => UniversalKeyCode { windows: 82, mac: 0x52 },
    // "KPDOT" => UniversalKeyCode { windows: 83, mac:  },
    // "84" => UniversalKeyCode { windows: 84, mac:  },
    // "F11" => UniversalKeyCode { windows: 87, mac:  },
    // "F12" => UniversalKeyCode { windows: 88, mac:  },
};
