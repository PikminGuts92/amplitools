use crate::apps::SubApp;
use amp_lib::bank::*;
use grim::ark::{Ark, ArkOffsetEntry};
use grim::io::{FileSearchDepth, PathFinder};
use clap::Parser;
use std::fmt::Debug;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::println;

#[derive(Parser, Debug)]
pub struct ExtractApp {
    #[arg(help = "Path to input amplitude ark (.ark)", required = true)]
    pub input_path: String,
    #[arg(help = "Path to output directory", required = true)]
    pub output_path: String,
}

impl SubApp for ExtractApp {
    fn process(self) -> Result<(), Box<dyn std::error::Error>> {
        let ark = Ark::from_path(&self.input_path)?;
        let output_path = PathBuf::from(self.output_path);

        let entry_count = ark.entries.len();
        println!("Found {entry_count} entries in ark!");

        /*let filtered_entries = ark
            .entries
            .iter()
            .filter(|e| e.path.ends_with(".bin"))
            .collect::<Vec<_>>();*/

        for entry in /*filtered_entries {*/ ark.entries.iter() {
            //std::path::MAIN_SEPARATOR_STR

            // Open stream
            let stream = ark.get_stream(entry.id)?;

            let (stream, entry_path) = match &entry.path {
                p @ _ if p.ends_with(".txt.bin") => {
                    // Convert bin to txt
                    let mut mem_stream = grim::io::MemoryStream::from_slice_as_read(&stream);
                    let mut reader = Box::new(grim::io::BinaryStream::from_stream_with_endian(&mut mem_stream, grim::io::IOEndian::Little));

                    let mut dta = grim::dta::RootData::new();
                    dta.load_with_settings(&mut reader, grim::dta::DataArrayIOSettings::Amplitude)?;

                    let mut writer = std::io::BufWriter::new(Vec::new());
                    dta.print(&mut writer)?;

                    // Write txt to file
                    //let txt = std::str::from_utf8(writer.buffer())?;
                    //println!("{txt}");

                    // Update file path
                    // - Remove .bin
                    // - Replace gen/
                    // TODO: Clean this up
                    let new_file_path = p[..(p.len() - 4)].replace("gen/", "");

                    (writer.into_inner()?, new_file_path)
                },
                _ => {
                    (stream, entry.path.to_owned())
                }
            };

            let file_path = output_path.join(&entry_path);

            // Write to file
            let mut file = grim::io::create_new_file(&file_path)?;
            file.write_all(&stream)?;
            println!("Wrote \"{}\"", &entry_path);

            //println!("Wrote \"{:?}\"", file_path);
        }

        println!("Done!");

        Ok(())
    }
}