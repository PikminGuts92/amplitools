// Hide console if release build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use amp_lib::bank::*;
use app::*;
use eframe::{NativeOptions, run_native};
use pikaxe::io::{FileSearchDepth, PathFinder};
use pikaxe::midi::{MidiEvent, MidiFile, MidiText, MidiTextType};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

// From Cargo.toml
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() < 1 {
        return Ok(());
    }

    let dir_path = Path::new(&args[0]);
    let mut app = AmpApp::default();

    app.open_directory(dir_path.into());

    let ops = NativeOptions {
        // icon_data: Some(icon),
        viewport: eframe::egui::ViewportBuilder::default()
            .with_min_inner_size([1000., 600.]) // min_window_size?
            .with_drag_and_drop(true),
        //#[cfg(feature = "dev")] initial_window_pos: Some([2400., 100.].into()),
        //#[cfg(feature = "dev")] always_on_top: true,
        ..NativeOptions::default()
    };

    run_native(
        "Amped by PikminGuts92",
        ops,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());

            Ok(Box::new(app))
        })
    )
    .map_err(|e| e.into())
}
