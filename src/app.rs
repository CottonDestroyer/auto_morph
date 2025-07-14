#[cfg(target_os = "macos")]
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult,
};
use eframe::egui;
use egui_file_dialog::FileDialog;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
#[cfg(target_os = "windows")]
use rdev::{Event, EventType};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use copypasta::{ClipboardContext, ClipboardProvider};

static IS_SIMULATING: AtomicBool = AtomicBool::new(false);

// --- Helper functions to make hotkey display readable ---

#[cfg(target_os = "macos")]
/// Converts macOS modifier flags into a vector of readable strings.
fn flags_to_strings(flags: CGEventFlags) -> Vec<String> {
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

#[cfg(target_os = "macos")]
/// Converts a macOS keycode into a readable string.
fn keycode_to_string(keycode: u64) -> String {
    match keycode {
        0 => "A", 1 => "S", 2 => "D", 3 => "F", 4 => "H", 5 => "G", 6 => "Z", 7 => "X",
        8 => "C", 9 => "V", 11 => "B", 12 => "Q", 13 => "W", 14 => "E", 15 => "R",
        16 => "Y", 17 => "T", 18 => "1", 19 => "2", 20 => "3", 21 => "4", 22 => "6",
        23 => "5", 24 => "=", 25 => "9", 26 => "7", 27 => "-", 28 => "8", 29 => "0",
        30 => "]", 31 => "O", 32 => "U", 33 => "[", 34 => "I", 35 => "P", 36 => "Enter",
        37 => "L", 38 => "J", 39 => "'", 40 => "K", 41 => ";", 42 => "\\", 43 => ",",
        44 => "/", 45 => "N", 46 => "M", 47 => ".", 48 => "Tab", 49 => "Space",
        50 => "`", 51 => "Delete", 53 => "Escape",
        55 => "Cmd", 56 => "Shift", 57 => "CapsLock", 58 => "Option", 59 => "Ctrl",
        60 => "Right Shift", 61 => "Right Option", 62 => "Right Ctrl",
        123 => "Left Arrow", 124 => "Right Arrow", 125 => "Down Arrow", 126 => "Up Arrow",
        _ => "Unknown",
    }.to_string()
}

#[cfg(target_os = "windows")]
/// Cleans up the debug-style output from rdev::Key.
fn key_to_string(key: &rdev::Key) -> String {
    format!("{:?}", key)
        .replace("Key", "")
        .replace("Left", " L")
        .replace("Right", " R")
}

fn log_message(log: &Arc<Mutex<Vec<String>>>, message: &str, ctx: &egui::Context) {
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

fn commands(cmds: &str, delay: u64, log: Arc<Mutex<Vec<String>>>, ctx: &egui::Context) {
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

        log_message(&log, &format!("Morphing line: {}", line), ctx);

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

    IS_SIMULATING.store(false, Ordering::SeqCst);
    log_message(&log, "Morph process finished.", ctx);
}

pub struct App {
    txt_cmds: String,
    cmds: Arc<Mutex<String>>,
    delay: Arc<Mutex<String>>,
    file_dialog: FileDialog,
    file: Option<PathBuf>,
    debug_log: Arc<Mutex<Vec<String>>>,
    #[cfg(target_os = "windows")]
    hotkey: Arc<Mutex<HashSet<rdev::Key>>>,
    #[cfg(target_os = "macos")]
    hotkey: Arc<Mutex<(HashSet<u64>, CGEventFlags)>>,
    is_capturing_hotkey: Arc<Mutex<bool>>,
    hotkey_display_text: String,
    hotkey_text_receiver: mpsc::Receiver<String>,
    hotkey_text_sender: mpsc::Sender<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let (key_tx, key_rx) = mpsc::channel();
        let (hotkey_text_tx, hotkey_text_rx) = mpsc::channel();
        let debug_log = Arc::new(Mutex::new(Vec::new()));
        let ctx = cc.egui_ctx.clone();

        let app = Self {
            cmds: Arc::new(Mutex::new(String::new())),
            txt_cmds: String::new(),
            delay: Arc::new(Mutex::new("40".to_owned())),
            file_dialog: FileDialog::new(),
            file: None,
            debug_log: Arc::clone(&debug_log),
            #[cfg(target_os = "windows")]
            hotkey: Arc::new(Mutex::new(HashSet::from([rdev::Key::ShiftRight]))),
            #[cfg(target_os = "macos")]
            hotkey: Arc::new(Mutex::new((
                HashSet::new(),
                CGEventFlags::CGEventFlagShift,
            ))),
            is_capturing_hotkey: Arc::new(Mutex::new(false)),
            hotkey_display_text: "Right Shift".to_string(),
            hotkey_text_receiver: hotkey_text_rx,
            hotkey_text_sender: hotkey_text_tx.clone(),
        };

        app.listen(key_tx, Arc::clone(&app.debug_log), ctx.clone(), hotkey_text_tx);

        let cmds_clone = Arc::clone(&app.cmds);
        let delay_clone = Arc::clone(&app.delay);
        let log_clone = Arc::clone(&app.debug_log);
        let logic_ctx = ctx;

        std::thread::spawn(move || {
            for _ in key_rx {
                if IS_SIMULATING.load(Ordering::SeqCst) {
                    log_message(
                        &log_clone,
                        "Simulation already in progress, ignoring hotkey.",
                        &logic_ctx,
                    );
                    continue;
                }
                let cmds = cmds_clone.lock().unwrap().clone();
                let delay = delay_clone.lock().unwrap().parse::<u64>().unwrap_or(40);
                let thread_log = Arc::clone(&log_clone);
                let thread_ctx = logic_ctx.clone();
                log_message(
                    &log_clone,
                    "Hotkey signal received, spawning a new simulation thread.",
                    &logic_ctx,
                );
                std::thread::spawn(move || {
                    App::morph(&cmds, delay, thread_log, &thread_ctx);
                });
            }
        });

        app
    }

    #[cfg(target_os = "macos")]
    fn listen(&self, tx: mpsc::Sender<()>, log: Arc<Mutex<Vec<String>>>, ctx: egui::Context, hotkey_text_sender: mpsc::Sender<String>) {
        let is_capturing_hotkey = Arc::clone(&self.is_capturing_hotkey);
        let hotkey_arc = Arc::clone(&self.hotkey);

        std::thread::spawn(move || {
            log_message(&log, "Starting key listener thread...", &ctx);

            let pressed_keys = Arc::new(Mutex::new(HashSet::<u64>::new()));
            let pressed_flags = Arc::new(Mutex::new(CGEventFlags::empty()));
            let hotkey_is_down = Arc::new(AtomicBool::new(false));
            let temp_capture_keys = Arc::new(Mutex::new(HashSet::<u64>::new()));
            let temp_capture_flags = Arc::new(Mutex::new(CGEventFlags::empty()));

            let callback_log = Arc::clone(&log);
            let callback_ctx = ctx.clone();
            let callback = move |_, event_type, event: &CGEvent| -> CallbackResult {
                let mut is_capturing = is_capturing_hotkey.lock().unwrap();

                if *is_capturing {
                    match event_type {
                        CGEventType::KeyDown => {
                            let keycode = event.get_integer_value_field(9);
                            temp_capture_keys.lock().unwrap().insert(keycode as u64);
                            *temp_capture_flags.lock().unwrap() = event.get_flags();
                        }
                        CGEventType::FlagsChanged => {
                            *temp_capture_flags.lock().unwrap() = event.get_flags();
                        }
                        CGEventType::KeyUp => {
                            let final_keys = temp_capture_keys.lock().unwrap().clone();
                            let final_flags = temp_capture_flags.lock().unwrap().clone();
                            if !final_keys.is_empty() || !final_flags.is_empty() {
                                let mut hotkey = hotkey_arc.lock().unwrap();
                                *hotkey = (final_keys.clone(), final_flags);
                                let mut display_parts = flags_to_strings(final_flags);
                                let non_modifier_keys: Vec<String> = final_keys.iter().filter(|&&k| !(55..=63).contains(&k)).map(|&k| keycode_to_string(k)).collect();
                                display_parts.extend(non_modifier_keys);
                                let display_text = display_parts.join(" + ");
                                let _ = hotkey_text_sender.send(display_text);
                            }
                            temp_capture_keys.lock().unwrap().clear();
                            *temp_capture_flags.lock().unwrap() = CGEventFlags::empty();
                            *is_capturing = false;
                            return CallbackResult::Keep;
                        }
                        _ => {}
                    }
                    let mut display_parts = flags_to_strings(*temp_capture_flags.lock().unwrap());
                     let non_modifier_keys: Vec<String> = temp_capture_keys.lock().unwrap().iter().filter(|&&k| !(55..=63).contains(&k)).map(|&k| keycode_to_string(k)).collect();
                    display_parts.extend(non_modifier_keys);
                    let _ = hotkey_text_sender.send(display_parts.join(" + "));
                    return CallbackResult::Keep;
                }

                match event_type {
                    CGEventType::KeyDown => {
                        pressed_keys.lock().unwrap().insert(event.get_integer_value_field(9) as u64);
                    }
                    CGEventType::KeyUp => {
                        pressed_keys.lock().unwrap().remove(&(event.get_integer_value_field(9) as u64));
                    }
                    CGEventType::FlagsChanged => {
                        *pressed_flags.lock().unwrap() = event.get_flags();
                    }
                    _ => {}
                }
                let target_hotkey = hotkey_arc.lock().unwrap();
                let current_keys = pressed_keys.lock().unwrap();
                let current_flags = *pressed_flags.lock().unwrap();
                let keys_match = *current_keys == target_hotkey.0;
                let all_target_flags_pressed = current_flags.contains(target_hotkey.1);
                if keys_match && all_target_flags_pressed {
                    if !hotkey_is_down.load(Ordering::SeqCst) {
                        log_message(&callback_log, "Hotkey PRESSED!", &callback_ctx);
                        let _ = tx.send(());
                        hotkey_is_down.store(true, Ordering::SeqCst);
                    }
                } else {
                    hotkey_is_down.store(false, Ordering::SeqCst);
                }
                CallbackResult::Keep
            };
            if let Ok(tap) = CGEventTap::new(CGEventTapLocation::HID, CGEventTapPlacement::HeadInsertEventTap, CGEventTapOptions::Default, vec![CGEventType::KeyDown, CGEventType::KeyUp, CGEventType::FlagsChanged], callback) {
                log_message(&log, "Event tap created successfully.", &ctx);
                unsafe {
                    let loop_source = tap.mach_port().create_runloop_source(0).unwrap();
                    CFRunLoop::get_current().add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                }
            } else {
                log_message(&log, "Failed to create event tap. Check macOS permissions.", &ctx);
            }
        });
    }

    #[cfg(target_os = "windows")]
    fn listen(&self, tx: mpsc::Sender<()>, log: Arc<Mutex<Vec<String>>>, ctx: egui::Context, hotkey_text_sender: mpsc::Sender<String>) {
        let is_capturing_hotkey = Arc::clone(&self.is_capturing_hotkey);
        let hotkey_arc = Arc::clone(&self.hotkey);
        let log_clone = Arc::clone(&log);
        let ctx_clone = ctx.clone();

        std::thread::spawn(move || {
            log_message(&log_clone, "Starting Windows key listener thread...", &ctx_clone);
            let pressed_keys = Arc::new(Mutex::new(HashSet::<rdev::Key>::new()));
            let mut temp_capture_set = HashSet::new();

            let callback = move |event: Event| {
                let mut p_keys = pressed_keys.lock().unwrap();
                let mut is_capturing = is_capturing_hotkey.lock().unwrap();

                match event.event_type {
                    EventType::KeyPress(key) => {
                        p_keys.insert(key);
                        if *is_capturing {
                            temp_capture_set.extend(p_keys.iter().cloned());
                            let key_names: Vec<String> = temp_capture_set.iter().map(key_to_string).collect();
                            let _ = hotkey_text_sender.send(key_names.join(" + "));
                        } else {
                            let target_hotkey = hotkey_arc.lock().unwrap();
                            if !target_hotkey.is_empty() && *target_hotkey == *p_keys {
                                log_message(&log_clone, "Hotkey PRESSED!", &ctx_clone);
                                let _ = tx.send(());
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if *is_capturing {
                            if !temp_capture_set.is_empty() {
                                let mut final_hotkey = hotkey_arc.lock().unwrap();
                                *final_hotkey = temp_capture_set.clone();
                                let key_names: Vec<String> = final_hotkey.iter().map(key_to_string).collect();
                                let display_text = key_names.join(" + ");
                                let _ = hotkey_text_sender.send(display_text);
                                temp_capture_set.clear();
                                *is_capturing = false;
                            }
                        }
                        p_keys.remove(&key);
                    }
                    _ => {}
                }
            };
            if let Err(error) = rdev::listen(callback) {
                log_message(&log, &format!("Error listening to keyboard: {:?}", error), &ctx);
            }
        });
    }

    fn morph(cmds: &str, delay: u64, log: Arc<Mutex<Vec<String>>>, ctx: &egui::Context) {
        if cmds.is_empty() {
            log_message(&log, "Morph commands are empty, skipping.", ctx);
            return;
        }
        log_message(&log, "Starting morph process...", ctx);
        IS_SIMULATING.store(true, Ordering::SeqCst);
        commands(cmds, delay, log, ctx);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(new_text) = self.hotkey_text_receiver.try_recv() {
            self.hotkey_display_text = new_text;
        }

        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.heading("SCP:RP Auto Morpher 🎯");
        });

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.with_layout(
                            egui::Layout::top_down_justified(egui::Align::Center),
                            |ui| { ui.heading("Instructions"); },
                        );
                        ui.separator();
                        ui.label("1. Paste morph in textbox, or pick a file.");
                        ui.label("2. Click 'Set Morph' button.");
                        ui.label("3. (Optional) Set a custom hotkey.");
                        ui.label("4. Press the hotkey while in-game.");
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    if ui.add_sized([ui.available_width(), 60.0], egui::Button::new("Set Morph")).clicked() {
                        log_message(&self.debug_log, "'Set Morph' button clicked.", ctx);
                        *self.cmds.lock().unwrap() = self.txt_cmds.clone();
                    }

                    ui.add_space(5.0);

                    if ui.add_sized([ui.available_width(), 40.0], egui::Button::new("Reset")).clicked() {
                        let reset_text = "unpermall me\nunpermhats me\nunpermshirt me\nclearstartergear me".to_owned();
                        self.txt_cmds = reset_text.clone();
                        *self.cmds.lock().unwrap() = reset_text;
                        log_message(&self.debug_log, "Commands reset to default.", ctx);
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label("File:");
                    if ui.add(egui::Button::new("Pick File")).clicked() {
                        self.file_dialog.pick_file();
                    }
                    if let Some(path) = &self.file {
                        if let Some(name) = path.file_name() {
                            ui.label(format!("Chosen: {}", name.to_string_lossy()));
                        }
                    } else {
                         ui.label("No file chosen");
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label("Delay (ms):");
                    ui.text_edit_singleline(&mut *self.delay.lock().unwrap());

                    ui.add_space(10.0);
                    
                    let mut is_capturing = self.is_capturing_hotkey.lock().unwrap();
                    let button_text = if *is_capturing { "Recording..." } else { "Set Hotkey" };
                    if ui.add(egui::Button::new(button_text)).clicked() && !*is_capturing {
                        *is_capturing = true;
                        self.hotkey_display_text.clear();
                        let _ = self.hotkey_text_sender.send("Recording...".to_string());
                        log_message(&self.debug_log, "Started capturing new hotkey.", ctx);
                    }
                    ui.label(format!("Hotkey: {}", &self.hotkey_display_text));
                });
            });

        // **FIX**: The file dialog update and result handling must be called every frame.
        self.file_dialog.update(ctx);
        if let Some(path) = self.file_dialog.take_picked() {
            log_message(&self.debug_log, &format!("File picked: {:?}", path), ctx);
            self.file = Some(path.to_path_buf());
            if let Ok(mut file) = fs::File::open(self.file.clone().unwrap()) {
                self.txt_cmds = String::new();
                let _ = file.read_to_string(&mut self.txt_cmds);
                log_message(&self.debug_log, "Read file into textbox.", ctx);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(target_os = "macos")]
            {
                ui.label("Note for mac users: If hotkeys don't work, grant Accessibility & Input Monitoring permissions in System Settings and restart the app.");
                ui.separator();
            }

            ui.add(
                egui::TextEdit::multiline(&mut self.txt_cmds)
                    .desired_width(f32::INFINITY)
                    .desired_rows(10),
            );

            ui.separator();

            ui.heading("Debug Log");
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let log = self.debug_log.lock().unwrap();
                    for msg in log.iter() {
                        ui.label(msg);
                    }
                });
        });
    }
}
