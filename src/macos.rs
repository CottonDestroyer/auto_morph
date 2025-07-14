use core_graphics::event::CGEventFlags;

pub fn flags_to_strings(flags: CGEventFlags) -> Vec<String> {
    let mut parts = Vec::new();
    if flags.contains(CGEventFlags::CGEventFlagCommand) {
        parts.push("Cmd".to_string());
    }
    if flags.contains(CGEventFlags::CGEventFlagShift) {
        parts.push("Shift".to_string());
    }
    if flags.contains(CGEventFlags::CGEventFlagControl) {
        parts.push("Ctrl".to_string());
    }
    if flags.contains(CGEventFlags::CGEventFlagAlternate) {
        parts.push("Option".to_string());
    }
    parts
}

pub fn keycode_to_string(keycode: u64) -> String {
    match keycode {
        0 => "A",
        1 => "S",
        2 => "D",
        3 => "F",
        4 => "H",
        5 => "G",
        6 => "Z",
        7 => "X",
        8 => "C",
        9 => "V",
        11 => "B",
        12 => "Q",
        13 => "W",
        14 => "E",
        15 => "R",
        16 => "Y",
        17 => "T",
        18 => "1",
        19 => "2",
        20 => "3",
        21 => "4",
        22 => "6",
        23 => "5",
        24 => "=",
        25 => "9",
        26 => "7",
        27 => "-",
        28 => "8",
        29 => "0",
        30 => "]",
        31 => "O",
        32 => "U",
        33 => "[",
        34 => "I",
        35 => "P",
        36 => "Enter",
        37 => "L",
        38 => "J",
        39 => "'",
        40 => "K",
        41 => ";",
        42 => "\\",
        43 => ",",
        44 => "/",
        45 => "N",
        46 => "M",
        47 => ".",
        48 => "Tab",
        49 => "Space",
        50 => "`",
        51 => "Delete",
        53 => "Escape",
        55 => "Cmd",
        56 => "Shift",
        57 => "CapsLock",
        58 => "Option",
        59 => "Ctrl",
        60 => "Right Shift",
        61 => "Right Option",
        62 => "Right Ctrl",
        123 => "Left Arrow",
        124 => "Right Arrow",
        125 => "Down Arrow",
        126 => "Up Arrow",
        _ => "Unknown",
    }
    .to_string()
}
