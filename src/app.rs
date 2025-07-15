use crate::utils::{commands, log_message};

#[cfg(target_os = "macos")]
use {
    crate::macos::{flags_to_strings, keycode_to_string},
    core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes},
    core_graphics::event::{
        CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventType, CallbackResult,
        bitflags::Flags,
        core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes},
        core_graphics::event::{
            CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
            CGEventTapPlacement, CGEventType, CallbackResult,
        },
    },
};

#[cfg(target_os = "windows")]
use {
    crate::windows::key_to_string,
    rdev::{Event, EventType},
};

use eframe::egui;
use egui_file_dialog::FileDialog;
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

static IS_SIMULATING: AtomicBool = AtomicBool::new(false);

fn morph(cmds: &str, delay: u64, log: Arc<Mutex<Vec<String>>>, ctx: &egui::Context) {
    if cmds.is_empty() {
        log_message(&log, "Morph commands are empty, skipping.", ctx);
        return;
    }
    log_message(&log, "Starting morph process...", ctx);
    IS_SIMULATING.store(true, Ordering::SeqCst);
    commands(cmds, delay, log, ctx);
    IS_SIMULATING.store(false, Ordering::SeqCst);
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
            hotkey: Arc::new(Mutex::new((HashSet::new(), CGEventFlags::CGEventFlagShift))),
            is_capturing_hotkey: Arc::new(Mutex::new(false)),
            hotkey_display_text: "Right Shift".to_string(),
            hotkey_text_receiver: hotkey_text_rx,
            hotkey_text_sender: hotkey_text_tx.clone(),
        };

        app.listen(
            key_tx,
            Arc::clone(&app.debug_log),
            ctx.clone(),
            hotkey_text_tx,
        );

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
                    morph(&cmds, delay, thread_log, &thread_ctx);
                });
            }
        });

        app
    }

    #[cfg(target_os = "macos")]
    fn listen(
        &self,
        tx: mpsc::Sender<()>,
        log: Arc<Mutex<Vec<String>>>,
        ctx: egui::Context,
        hotkey_text_sender: mpsc::Sender<String>,
    ) {
        let is_capturing_hotkey = self.is_capturing_hotkey.clone();
        let hotkey_arc = self.hotkey.clone();
        let callback_log = log.clone();
        let callback_ctx = ctx.clone();

        std::thread::spawn(move || {
            log_message(&log, "Starting key listener thread...", &ctx);

            let pressed_keys = Arc::new(Mutex::new(HashSet::<u64>::new()));
            let pressed_flags = Arc::new(Mutex::new(CGEventFlags::empty()));
            let hotkey_is_down = Arc::new(AtomicBool::new(false));

            let temp_capture_keys = Arc::new(Mutex::new(HashSet::<u64>::new()));
            let temp_capture_flags = Arc::new(Mutex::new(CGEventFlags::empty()));

            let update_display = move |keys: &HashSet<u64>, flags: CGEventFlags| {
                let mut key_parts = flags_to_strings(flags);
                key_parts.extend(keys.iter().map(|&k| keycode_to_string(k)));
                key_parts.sort_unstable();
                key_parts.dedup();
                let _ = hotkey_text_sender.send(key_parts.join(" + "));
            };

            let callback = move |_, event_type, event: &CGEvent| {
                let mut is_capturing = is_capturing_hotkey.lock().unwrap();

                if *is_capturing {
                    let mut temp_keys = temp_capture_keys.lock().unwrap();
                    let mut temp_flags = temp_capture_flags.lock().unwrap();
                    match event_type {
                        CGEventType::KeyDown => {
                            let keycode = event.get_integer_value_field(9) as u64;
                            temp_keys.insert(keycode);
                            *temp_flags = event.get_flags();
                        }
                        CGEventType::FlagsChanged => {
                            *temp_flags = event.get_flags();
                        }
                        CGEventType::KeyUp => {
                            if !temp_keys.is_empty() {
                                let mut final_hotkey = hotkey_arc.lock().unwrap();
                                *final_hotkey = (temp_keys.clone(), *temp_flags);
                                update_display(&temp_keys, *temp_flags);
                            }
                            temp_keys.clear();
                            temp_flags.clear();
                            *is_capturing = false;
                        }
                        _ => {}
                    }
                    if *is_capturing {
                        update_display(&temp_keys, *temp_flags);
                    }
                    return CallbackResult::Keep;
                }

                drop(is_capturing);
                let keycode = event.get_integer_value_field(9) as u64;
                let current_flags = event.get_flags();

                let mut keys = pressed_keys.lock().unwrap();
                let mut flags = pressed_flags.lock().unwrap();

                match event_type {
                    CGEventType::KeyDown => {
                        keys.insert(keycode);
                        *flags = current_flags;
                    }
                    CGEventType::KeyUp => {
                        keys.remove(&keycode);
                        *flags = current_flags;
                    }
                    CGEventType::FlagsChanged => {
                        *flags = current_flags;
                    }
                    _ => return CallbackResult::Keep,
                }

                let target = hotkey_arc.lock().unwrap();
                let hotkey_is_met = keys.is_superset(&target.0) && flags.contains(target.1);

                if hotkey_is_met {
                    if hotkey_is_down
                        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
                        .is_ok()
                    {
                        log_message(&callback_log, "Hotkey PRESSED!", &callback_ctx);
                        let _ = tx.send(());
                    }
                } else {
                    hotkey_is_down.store(false, Ordering::SeqCst);
                }

                CallbackResult::Keep
            };

            let events = vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
            ];
            if let Ok(tap) = CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                events,
                callback,
            ) {
                log_message(&log, "Event tap created successfully.", &ctx);
                unsafe {
                    let loop_source = tap.mach_port().create_runloop_source(0).unwrap();
                    CFRunLoop::get_current().add_source(&loop_source, kCFRunLoopCommonModes);
                    tap.enable();
                    CFRunLoop::run_current();
                }
            } else {
                log_message(
                    &log,
                    "Failed to create event tap. Check macOS permissions.",
                    &ctx,
                );
            }
        });
    }

    #[cfg(target_os = "windows")]
    fn listen(
        &self,
        tx: mpsc::Sender<()>,
        log: Arc<Mutex<Vec<String>>>,
        ctx: egui::Context,
        hotkey_text_sender: mpsc::Sender<String>,
    ) {
        let is_capturing_hotkey = Arc::clone(&self.is_capturing_hotkey);
        let hotkey_arc = Arc::clone(&self.hotkey);
        let log_clone = Arc::clone(&log);
        let ctx_clone = ctx.clone();

        std::thread::spawn(move || {
            log_message(
                &log_clone,
                "Starting Windows key listener thread...",
                &ctx_clone,
            );
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
                            let key_names: Vec<String> =
                                temp_capture_set.iter().map(key_to_string).collect();
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
                        if *is_capturing && !temp_capture_set.is_empty() {
                            let mut final_hotkey = hotkey_arc.lock().unwrap();
                            *final_hotkey = temp_capture_set.clone();
                            let key_names: Vec<String> =
                                final_hotkey.iter().map(key_to_string).collect();
                            let display_text = key_names.join(" + ");
                            let _ = hotkey_text_sender.send(display_text);
                            temp_capture_set.clear();
                            *is_capturing = false;
                        }
                        p_keys.remove(&key);
                    }
                    _ => {}
                }
            };
            if let Err(error) = rdev::listen(callback) {
                log_message(
                    &log,
                    &format!("Error listening to keyboard: {error:?}"),
                    &ctx,
                );
            }
        });
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
                            |ui| {
                                ui.heading("Instructions");
                            },
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

                    if ui
                        .add_sized([ui.available_width(), 60.0], egui::Button::new("Set Morph"))
                        .clicked()
                    {
                        log_message(&self.debug_log, "'Set Morph' button clicked.", ctx);
                        *self.cmds.lock().unwrap() = self.txt_cmds.clone();
                    }

                    ui.add_space(5.0);

                    if ui
                        .add_sized([ui.available_width(), 40.0], egui::Button::new("Reset"))
                        .clicked()
                    {
                        let reset_text =
                            "unpermall me\nunpermhats me\nunpermshirt me\nclearstartergear me"
                                .to_owned();
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
                    let button_text = if *is_capturing {
                        "Recording..."
                    } else {
                        "Set Hotkey"
                    };
                    if ui.add(egui::Button::new(button_text)).clicked() && !*is_capturing {
                        *is_capturing = true;
                        self.hotkey_display_text.clear();
                        let _ = self.hotkey_text_sender.send("Recording...".to_string());
                        log_message(&self.debug_log, "Started capturing new hotkey.", ctx);
                    }
                    ui.label(format!("Hotkey: {}", &self.hotkey_display_text));
                });
            });

        self.file_dialog.update(ctx);
        if let Some(path) = self.file_dialog.take_picked() {
            log_message(&self.debug_log, &format!("File picked: {path:?}"), ctx);
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
                ui.label("Note for mac users: If hotkeys don't work, grant Accessibility permissions in System Settings and restart the app.");
                ui.separator();
            }

            ui.add(egui::TextEdit::multiline(&mut self.txt_cmds)
                .font(egui::FontId::new(16.0, egui::FontFamily::Monospace))
                .desired_width(f32::INFINITY)
                .desired_rows(20),
        );

            ui.separator();

            ui.heading("Debug Logs");
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
