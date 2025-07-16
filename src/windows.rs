use egui_keybinds::KeyCode as EKey;
use rdev::Key as RKey;

pub fn key_to_string(key: &rdev::Key) -> String {
    format!("{key:?}")
        .replace("Key", "")
        .replace("Left", " L")
        .replace("Right", " R")
}

pub fn egui_key_to_rdev_key(egui_key: EKey) -> Option<RKey> {
    match egui_key {
        EKey::F1 => Some(RKey::F1),
        EKey::F2 => Some(RKey::F2),
        EKey::F3 => Some(RKey::F3),
        EKey::F4 => Some(RKey::F4),
        EKey::F5 => Some(RKey::F5),
        EKey::F6 => Some(RKey::F6),
        EKey::F7 => Some(RKey::F7),
        EKey::F8 => Some(RKey::F8),
        EKey::F9 => Some(RKey::F9),
        EKey::F10 => Some(RKey::F10),
        EKey::F11 => Some(RKey::F11),
        EKey::F12 => Some(RKey::F12),

        EKey::Num0 => Some(RKey::Num0),
        EKey::Num1 => Some(RKey::Num1),
        EKey::Num2 => Some(RKey::Num2),
        EKey::Num3 => Some(RKey::Num3),
        EKey::Num4 => Some(RKey::Num4),
        EKey::Num5 => Some(RKey::Num5),
        EKey::Num6 => Some(RKey::Num6),
        EKey::Num7 => Some(RKey::Num7),
        EKey::Num8 => Some(RKey::Num8),
        EKey::Num9 => Some(RKey::Num9),

        EKey::A => Some(RKey::KeyA),
        EKey::B => Some(RKey::KeyB),
        EKey::C => Some(RKey::KeyC),
        EKey::D => Some(RKey::KeyD),
        EKey::E => Some(RKey::KeyE),
        EKey::F => Some(RKey::KeyF),
        EKey::G => Some(RKey::KeyG),
        EKey::H => Some(RKey::KeyH),
        EKey::I => Some(RKey::KeyI),
        EKey::J => Some(RKey::KeyJ),
        EKey::K => Some(RKey::KeyK),
        EKey::L => Some(RKey::KeyL),
        EKey::M => Some(RKey::KeyM),
        EKey::N => Some(RKey::KeyN),
        EKey::O => Some(RKey::KeyO),
        EKey::P => Some(RKey::KeyP),
        EKey::Q => Some(RKey::KeyQ),
        EKey::R => Some(RKey::KeyR),
        EKey::S => Some(RKey::KeyS),
        EKey::T => Some(RKey::KeyT),
        EKey::U => Some(RKey::KeyU),
        EKey::V => Some(RKey::KeyV),
        EKey::W => Some(RKey::KeyW),
        EKey::X => Some(RKey::KeyX),
        EKey::Y => Some(RKey::KeyY),
        EKey::Z => Some(RKey::KeyZ),

        EKey::Escape => Some(RKey::Escape),
        EKey::Tab => Some(RKey::Tab),
        EKey::Backspace => Some(RKey::Backspace),
        EKey::Return => Some(RKey::Return),
        EKey::Space => Some(RKey::Space),
        EKey::CapsLock => Some(RKey::CapsLock),
        EKey::Insert => Some(RKey::Insert),
        EKey::Delete => Some(RKey::Delete),
        EKey::Home => Some(RKey::Home),
        EKey::End => Some(RKey::End),
        EKey::PageUp => Some(RKey::PageUp),
        EKey::PageDown => Some(RKey::PageDown),

        EKey::UpArrow => Some(RKey::UpArrow),
        EKey::DownArrow => Some(RKey::DownArrow),
        EKey::LeftArrow => Some(RKey::LeftArrow),
        EKey::RightArrow => Some(RKey::RightArrow),

        EKey::LShift => Some(RKey::ShiftLeft),
        EKey::RShift => Some(RKey::ShiftRight),
        EKey::LControl => Some(RKey::ControlLeft),
        EKey::RControl => Some(RKey::ControlRight),
        EKey::LAlt => Some(RKey::Alt),
        EKey::RAlt => Some(RKey::Alt),
        EKey::LWindows => Some(RKey::MetaLeft),
        EKey::RWindows => Some(RKey::MetaRight),
        EKey::LCommand => Some(RKey::MetaLeft),
        EKey::RCommand => Some(RKey::MetaRight),
        EKey::LOption => Some(RKey::Alt),
        EKey::ROption => Some(RKey::Alt),

        EKey::Backtick => Some(RKey::BackQuote),
        EKey::Minus => Some(RKey::Minus),
        EKey::Equals => Some(RKey::Equal),
        EKey::OpenBracket => Some(RKey::LeftBracket),
        EKey::CloseBracket => Some(RKey::RightBracket),
        EKey::Backslash => Some(RKey::BackSlash),
        EKey::SemiColon => Some(RKey::SemiColon),
        EKey::Apostrophe => Some(RKey::Quote),
        EKey::Comma => Some(RKey::Comma),
        EKey::Period => Some(RKey::Dot),
        EKey::ForwardSlash => Some(RKey::Slash),

        _ => None,
    }
}
