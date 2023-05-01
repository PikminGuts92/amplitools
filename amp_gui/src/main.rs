// Hide console if release build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use amp_lib::bank::*;
use app::*;
use eframe::{NativeOptions, run_native};
use grim::io::{FileSearchDepth, PathFinder};
use grim::midi::{MidiEvent, MidiFile, MidiText, MidiTextType};
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
        drag_and_drop_support: true,
        // icon_data: Some(icon),
        min_window_size: Some([1000., 600.].into()),
        follow_system_theme: false, // Always dark by default
        default_theme: eframe::Theme::Dark,
        //#[cfg(feature = "dev")] initial_window_pos: Some([2400., 100.].into()),
        //#[cfg(feature = "dev")] always_on_top: true,
        ..NativeOptions::default()
    };

    run_native(
        "Amped by PikminGuts92",
        ops,
        Box::new(|_cc| Box::new(app))
    )
    .map_err(|e| e.into())
}
