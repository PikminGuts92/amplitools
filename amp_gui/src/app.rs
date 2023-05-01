use amp_lib::bank::*;
use eframe::{egui::{self, Align, Align2, Color32, FontId, Pos2, RichText, Visuals, Widget, TextBuffer}, glow};
use grim::io::{FileSearchDepth, PathFinder};
use grim::midi::{MidiEvent, MidiFile, MidiText, MidiTextType};
use std::path::{Path, PathBuf};
use super::VERSION;

#[derive(Default)]
pub struct AmpApp {
    dir_path: Option<PathBuf>,
    bank_file: Option<BankFile>,
    selected_sample_index: usize,
}

impl AmpApp {
    fn reset_state(&mut self) {
        self.dir_path = None;
        self.bank_file = None;
        self.selected_sample_index = 0;
    }

    pub fn open_directory(&mut self, dir_path: PathBuf) {
        self.reset_state();

        let file_paths = dir_path.find_files_with_depth(FileSearchDepth::Immediate).unwrap();

        let mid_file_paths = file_paths
            .iter()
            .filter(|p| p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".mid"))) // Note: is_some_and is 1.70.0 feature
            .collect::<Vec<_>>();

        println!("Found {} files!", file_paths.len());
        println!("Found {} midi files!", mid_file_paths.len());

        for mp in mid_file_paths.iter() {
            let mf = MidiFile::from_path(mp).unwrap();

            let bank_track = mf.tracks
                .iter()
                .find(|t| t.name
                    .as_ref()
                    .is_some_and(|n| n.as_str().eq("BANK")))
                .unwrap();

            let bank_events = bank_track
                .events
                .iter()
                .flat_map(|e| match e {
                    MidiEvent::Meta(mt @ MidiText { pos_realtime: Some(pos), .. }) if mt.is_text()
                        => Some((*pos, mt.as_str().unwrap())),
                    _ => None
                })
                .collect::<Vec<_>>();

            println!("Found {} bank events", bank_events.len());

            /*for (pos, name) in bank_events.iter() {
                println!("{pos}ms: {name}");
            }*/

            // Open bank file
            if let Some((_, name)) = bank_events.first() {
                println!("Opening {name}");

                let bank_path = dir_path.join(name);
                let bank_file = BankFile::from_file(&bank_path).unwrap();

                println!("Found {} samples", bank_file.samples.len());
                print!("{bank_file:#?}");

                // Update bank file
                self.dir_path = Some(dir_path);
                self.bank_file = Some(bank_file);
            }

            break;
        }
    }
}

impl eframe::App for AmpApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use egui_extras::{Column, TableBuilder};

        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    //.columns(Column::auto(), 4)
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    //.column(Column::auto())
                    .column(Column::remainder())
                    .header(20., |mut header| {
                        header.col(|ui| { ui.strong("#"); });
                        header.col(|ui| { ui.strong("Name"); });
                        header.col(|ui| { ui.strong("Ch."); });
                        header.col(|ui| { ui.strong("Source"); });
                });

                let Some(bank) = self.bank_file.as_ref() else {
                    return
                };

                table.body(|mut body| {
                    for (i, sample) in bank.samples.iter().enumerate() {
                        body.row(18., |mut row| {
                            row.col(|ui| { ui.label(i.to_string()); });
                            row.col(|ui| { ui.label(sample.name.as_str()); });
                            row.col(|ui| { ui.label(sample.channels.to_string()); });
                            row.col(|ui| { ui.label(sample.file_name.as_str()); });
                        });
                    }
                });
            //});
        });
    }
}