use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::sync::{Arc, Mutex};

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
