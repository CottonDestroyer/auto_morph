use eframe::egui;
use egui_file_dialog::FileDialog;
use enigo::{Enigo, Keyboard, Settings};
use rdev;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static IS_SIMULATING: AtomicBool = AtomicBool::new(false); // if u have a better method please tell me ong

pub struct App {
    txt_cmds: String,
    cmds: Arc<Mutex<String>>,
    delay: Arc<Mutex<String>>,  // we love the mutexes..maybe should hav used RwLock?
    file_dialog: FileDialog,
    file: Option<PathBuf>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let app = Self {
            cmds: Arc::new(Mutex::new(String::new())),
            txt_cmds: String::new(),
            delay: Arc::new(Mutex::new("100".to_owned())),
            file_dialog: FileDialog::new(),
            file: None,
        };
        app.listen();
        app
    }

    fn listen(&self) {
        let cmds = self.cmds.clone();
        let delay = self.delay.clone();
        std::thread::spawn(move || {
            rdev::listen(move |event| {
                if IS_SIMULATING.load(Ordering::SeqCst) {
                    return;
                }
                if let rdev::EventType::KeyPress(key) = event.event_type {
                    if key == rdev::Key::ShiftRight {
                        let clock = cmds.lock().unwrap();
                        App::morph(       
                            clock.trim(),
                            delay.lock().unwrap().parse::<u64>().unwrap_or(100),     // this shit took me hours, had to make morph() a static method bruh
                        );
                    }
                }
            })
            .unwrap();
        });
    }

    fn morph(cmds: &str, delay: u64) {
        if cmds.is_empty() {
            return;
        }
        IS_SIMULATING.store(true, Ordering::SeqCst);
        let mut enigo = Enigo::new(&Settings::default()).unwrap();

        for line in cmds.lines() {
            if line.trim() == "" {
                continue;
            }
            enigo.raw(40, enigo::Direction::Click).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay));
            enigo
                .key(enigo::Key::Backspace, enigo::Direction::Click)
                .unwrap();
            enigo.text(line).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay));
            enigo
                .key(enigo::Key::Return, enigo::Direction::Click)
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        IS_SIMULATING.store(false, Ordering::SeqCst);
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
                        let mut clock = self.cmds.lock().unwrap();
                        *clock = self.txt_cmds.clone();
                    }

                    ui.text_edit_singleline(&mut *self.delay.lock().unwrap());
                    ui.label("Delay");

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
                    self.file = Some(path.to_path_buf());
                    let _ = fs::File::open(self.file.clone().unwrap())
                        .unwrap()
                        .read_to_string(&mut self.txt_cmds);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let free_size = ui.available_size();
            ui.add_sized(
                free_size,
                egui::TextEdit::multiline(&mut self.txt_cmds)
                    .frame(true)
                    .desired_rows(10),
            );
        });
    }
}
