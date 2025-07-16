use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::sync::{Arc, Mutex};
use egui_keybinds::KeyCode as EKey;
use rdev::Key as RKey;

pub fn commands(cmds: &str, delay: u64, log: Arc<Mutex<Vec<String>>>, ctx: &egui::Context) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let mut clipboard = ClipboardContext::new().expect("Failed to get clipboard context.");
    let original_clipboard = clipboard.get_contents().ok();

    for mut line in cmds.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if line.starts_with(':') {
            line = &line[1..];
        }

        log_message(&log, &format!("Morphing line: {line}"), ctx);

        #[cfg(target_os = "windows")]
        {
            enigo.raw(40, Direction::Click).unwrap()
        }
        #[cfg(target_os = "macos")]
        {
            enigo.raw(39, Direction::Click).unwrap()
        }
        std::thread::sleep(std::time::Duration::from_millis(delay));

        enigo.key(Key::Backspace, Direction::Click).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(delay / 2));

        copy_paste(line, &mut enigo, &mut clipboard);

        std::thread::sleep(std::time::Duration::from_millis(delay));

        enigo.key(Key::Return, Direction::Click).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(delay));
    }

    if let Some(original) = original_clipboard {
        if clipboard.set_contents(original).is_ok() {
            log_message(&log, "Clipboard restored.", ctx);
        }
    }
    log_message(&log, "Morph process finished.", ctx);
}

pub fn log_message(log: &Arc<Mutex<Vec<String>>>, message: &str, ctx: &egui::Context) {
    let mut log_guard = log.lock().unwrap();
    log_guard.push(message.to_owned());
    if log_guard.len() > 100 {
        log_guard.remove(0);
    }
    ctx.request_repaint();
}

fn copy_paste(text: &str, enigo: &mut Enigo, clipboard: &mut ClipboardContext) {
    clipboard.set_contents(text.to_owned()).unwrap();

    #[cfg(target_os = "windows")]
    {
        enigo.key(Key::Control, Direction::Press).unwrap();
        enigo.key(Key::V, Direction::Click).unwrap();
        enigo.key(Key::Control, Direction::Release).unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).unwrap();
        enigo.key(Key::Other(0x00000009), Direction::Click).unwrap();
        enigo.key(Key::Meta, Direction::Release).unwrap();
    }
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

        _ => None
    }
}