#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use auto_morph::app::App;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([700.0, 650.0])
            .with_resizable(true)
            .with_icon(
                eframe::icon_data::from_png_bytes(
                    &include_bytes!("../assets/rocket-lunch.png")[..],
                )
                .expect("Couldn't find icon u skibi"),
            ),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Blazingly fast SCP:RP morpher",
        options,
        Box::new(|cc| Ok(Box::new(<App>::new(cc)))),
    )
}
