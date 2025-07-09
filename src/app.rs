#[cfg(target_os = "macos")]
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType,
};
#[cfg(target_os = "windows")]
use rdev::{Event, EventType};
use eframe::egui;
use egui_file_dialog::FileDialog;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::cell::Cell;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use copypasta::{ClipboardProvider, ClipboardContext};


static IS_SIMULATING: AtomicBool = AtomicBool::new(false);

fn log_message(log: &Arc<Mutex<Vec<String>>>, message: &str, ctx: &egui::Context) {
    let mut log_guard = log.lock().unwrap();
    println!("{}", message);
    log_guard.push(message.to_owned());
    if log_guard.len() > 100 {
        log_guard.remove(0);
    }
    ctx.request_repaint();
}

fn copy_paste(text: &str, enigo: &mut Enigo) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(text.to_owned()).unwrap();

    #[cfg(target_os = "windows")] {
        enigo.key(Key::Control, Direction::Press).unwrap();
        enigo.key(Key::V, Direction::Click).unwrap();
        enigo.key(Key::Control, Direction::Release).unwrap();
    }

    #[cfg(target_os = "macos")] {
        enigo.key(Key::Meta, Direction::Press).unwrap();
        enigo.key(Key::Unicode('v'), Direction::Click).unwrap();
        enigo.key(Key::Meta, Direction::Release).unwrap();
    }
}

pub struct App {
    txt_cmds: String,
    cmds: Arc<Mutex<String>>,
    delay: Arc<Mutex<String>>,
    file_dialog: FileDialog,
    file: Option<PathBuf>,
    debug_log: Arc<Mutex<Vec<String>>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let (tx, key_receiver) = mpsc::channel();

        let debug_log = Arc::new(Mutex::new(Vec::new()));
        let ctx = cc.egui_ctx.clone();

        let app = Self {
            cmds: Arc::new(Mutex::new(String::new())),
            txt_cmds: String::new(),
            delay: Arc::new(Mutex::new("30".to_owned())),
            file_dialog: FileDialog::new(),
            file: None,
            debug_log: Arc::clone(&debug_log),
        };

        app.listen(tx, Arc::clone(&app.debug_log), ctx.clone());

        let cmds_clone = Arc::clone(&app.cmds);
        let delay_clone = Arc::clone(&app.delay);
        let log_clone = Arc::clone(&app.debug_log);
        let logic_ctx = ctx;

        std::thread::spawn(move || {
            for _ in key_receiver {
                log_message(
                    &log_clone,
                    "Hotkey signal received in logic thread.",
                    &logic_ctx,
                );
                if !IS_SIMULATING.load(Ordering::SeqCst) {
                    let cmds = cmds_clone.lock().unwrap();
                    let delay = delay_clone.lock().unwrap().parse::<u64>().unwrap_or(30);
                    App::morph(&cmds, delay, Arc::clone(&log_clone), &logic_ctx);
                } else {
                    log_message(
                        &log_clone,
                        "Simulation already in progress, ignoring hotkey.",
                        &logic_ctx,
                    );
                }
            }
        });

        app
    }
    #[cfg(target_os = "macos")]
    fn listen(&self, tx: mpsc::Sender<()>, log: Arc<Mutex<Vec<String>>>, ctx: egui::Context) {
        std::thread::spawn(move || {
            log_message(&log, "Starting key listener thread...", &ctx);

            let right_shift_is_down = Cell::new(false);

            let log_for_callback = Arc::clone(&log);
            let callback_ctx = ctx.clone(); // Clone the context for the callback.
            let callback = move |_, _, event: &CGEvent| -> Option<CGEvent> {
                let keycode = event.get_integer_value_field(9);

                if keycode == 60 {
                    let flags = event.get_flags();
                    let a_shift_key_is_pressed = flags.contains(CGEventFlags::CGEventFlagShift);

                    if a_shift_key_is_pressed && !right_shift_is_down.get() {
                        log_message(
                            &log_for_callback,
                            "Right Shift (hotkey) PRESSED!",
                            &callback_ctx,
                        );
                        let _ = tx.send(());
                    } else if !a_shift_key_is_pressed && right_shift_is_down.get() {
                        /*log_message(
                            &log_for_callback,
                            "Right Shift (hotkey) RELEASED!",
                            &callback_ctx,
                        );*/
                    }

                    right_shift_is_down.set(a_shift_key_is_pressed);
                }

                Some(event.to_owned())
            };

            let events_of_interest = vec![CGEventType::FlagsChanged];
            let tap = CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                events_of_interest,
                callback,
            );

            if let Ok(tap) = tap {
                log_message(&log, "Event tap created successfully.", &ctx);
                unsafe {
                    let loop_source = tap.mach_port.create_runloop_source(0).unwrap();

                    let current_loop = CFRunLoop::get_current();
                    current_loop.add_source(&loop_source, kCFRunLoopCommonModes);

                    tap.enable();

                    log_message(&log, "Running event loop...", &ctx);
                    CFRunLoop::run_current();
                }
            } else {
                log_message(
                    &log,
                    "Failed to create event tap. Check macOS permissions for Input Monitoring/Accessibility.",
                    &ctx,
                );
            }
        });
    }

    #[cfg(target_os = "windows")]
    fn listen(
        &self,
        tx: std::sync::mpsc::Sender<()>,
        log: Arc<Mutex<Vec<String>>>,
        ctx: egui::Context,
    ) {

        std::thread::spawn(move || {
            log_message(&log, "Starting Windows key listener thread...", &ctx);

            let pressed = Cell::new(false);

            let _ = rdev::listen(move |event: Event| {
                if let EventType::KeyPress(key) = event.event_type {
                    if key == rdev::Key::ShiftRight && !pressed.get() {
                        log_message(&log, "Right Shift (hotkey) PRESSED!", &ctx);
                        pressed.set(true);
                        let _ = tx.send(());
                    }
                } else if let EventType::KeyRelease(key) = event.event_type {
                    if key == rdev::Key::ShiftRight && pressed.get() {
                        //log_message(&log, "Right Shift (hotkey) RELEASED!", &ctx);
                        pressed.set(false);
                    }
                }
            });
        });
    }

    fn morph(cmds: &str, delay: u64, log: Arc<Mutex<Vec<String>>>, ctx: &egui::Context) {
        if cmds.is_empty() {
            log_message(&log, "Morph commands are empty, skipping.", ctx);
            return;
        }
        log_message(&log, "Starting morph process...", ctx);
        IS_SIMULATING.store(true, Ordering::SeqCst);
        let mut enigo = Enigo::new(&Settings::default()).unwrap();

        for line in cmds.lines() {
            if line.trim().is_empty() {
                continue;
            }
            log_message(&log, &format!("Morphing line: {}", line), ctx);

            #[cfg(target_os = "windows")] {enigo.raw(40, Direction::Click).unwrap()}
            #[cfg(target_os = "macos")] {enigo.raw(39, Direction::Click).unwrap()}
            std::thread::sleep(std::time::Duration::from_millis(delay));

            enigo.key(Key::Backspace, Direction::Click).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay/2));

            copy_paste(line, &mut enigo);

            std::thread::sleep(std::time::Duration::from_millis(delay));

            enigo.key(Key::Return, Direction::Click).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        IS_SIMULATING.store(false, Ordering::SeqCst);
        log_message(&log, "Morph process finished.", ctx);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.heading("SCP:RP Auto Morpher 🎯");
        });

        egui::SidePanel::right("right panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.allocate_ui(egui::Vec2::new(200.0, 0.0), |ui| {
                    ui.group(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.heading("Instructions");
                            },
                        );
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.label("1. Paste morph in textbox, or pick a file");
                            ui.label("2. Click 'Set Morph' button");
                            ui.label("3. Press the Right Shift key while in-game");
                        });
                    });
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(10.0);
                    let morph_button = ui.add_sized([200.0, 80.0], egui::Button::new("Set Morph"));
                    if morph_button.clicked() {
                        log_message(&self.debug_log, "'Set Morph' button clicked.", ctx);
                        let mut clock = self.cmds.lock().unwrap();
                        *clock = self.txt_cmds.clone();
                    }

                    ui.text_edit_singleline(&mut *self.delay.lock().unwrap());
                    ui.label("Delay (ms)");
                    ui.add_space(10.0);

                    let file_button = ui.add_sized([200.0, 30.0], egui::Button::new("Pick File"));

                    if file_button.clicked() {
                        self.file_dialog.pick_file();
                    }

                    self.file_dialog.update(ctx);

                    match &self.file {
                        Some(path) => {
                            if let Some(name) = path.file_name() {
                                ui.label(format!("File chosen: {}", name.to_string_lossy()));
                            }
                        }
                        None => {
                            ui.label("No file chosen");
                        }
                    }
                });

                if let Some(path) = self.file_dialog.take_picked() {
                    log_message(&self.debug_log, &format!("File picked: {:?}", path), ctx);
                    self.file = Some(path.to_path_buf());
                    if let Ok(mut file) = fs::File::open(self.file.clone().unwrap()) {
                        self.txt_cmds = String::new();
                        let _ = file.read_to_string(&mut self.txt_cmds);
                        log_message(
                            &self.debug_log,
                            "Successfully read file content into textbox.",
                            ctx,
                        );
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            /*ui.label("Note for mac users: If the hotkey doesn't work, grant Accessibility & Input Monitoring permissions in System Settings and restart the app.");
            ui.separator();*/

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
