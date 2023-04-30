use amp_lib::bank::*;
use grim::io::{FileSearchDepth, PathFinder};
use grim::midi::{MidiEvent, MidiFile, MidiText, MidiTextType};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() < 1 {
        return;
    }

    let dir_path = Path::new(&args[0]);
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
        }

        break;
    }
}
