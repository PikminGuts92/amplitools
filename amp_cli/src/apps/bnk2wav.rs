use crate::apps::SubApp;
use amp_lib::bank::*;
use grim::io::{FileSearchDepth, PathFinder};
use clap::Parser;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct Bnk2WavApp {
    #[arg(help = "Path to input amplitude sample bank (.bnk)", required = true)]
    pub input_path: String,
    #[arg(help = "Path to output directory", required = true)]
    pub output_path: String,
}

impl SubApp for Bnk2WavApp {
    fn process(self) -> Result<(), Box<dyn std::error::Error>> {
        let input_path = Path::new(&self.input_path);
        let output_path = Path::new(&self.output_path);

        let mut total_samples = 0;
        let mut bank_count = 0;

        if input_path.is_file() {
            let samples = extract_samples(input_path, output_path)?;
            total_samples += samples;
        } else {
            let bnk_paths = input_path.find_files_with_depth(FileSearchDepth::Immediate)?
                .into_iter()
                .filter(|p| p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".bnk"))) // Note: is_some_and is 1.70.0 feature
                .collect::<Vec<_>>();

            for bnk_path in bnk_paths {
                let local_output_path = output_path.join(bnk_path.file_stem().unwrap());

                let samples = extract_samples(bnk_path.as_path(), local_output_path.as_path())?;
                total_samples += samples;
                bank_count += 1;
            }
        }

        //println!("Wrote {} samples to \"{}\"", bnk.samples.len(), sample_file_path.display());
        println!("Extracted {} total samples from {} banks", total_samples, bank_count);

        Ok(())
    }
}

fn extract_samples(bank_path: &Path, output_path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let sample_file_path = bank_path
        .canonicalize()
        .map(|fp| fp.parent().unwrap().join(format!("{}.nse", bank_path.file_stem().unwrap().to_str().unwrap())))?;

    let bnk = BankFile::from_file(bank_path)?;
    bnk.extract_samples_to_dir(&sample_file_path, output_path)?;

    println!("Wrote {} samples to \"{}\"", bnk.samples.len(), sample_file_path.display());

    Ok(bnk.samples.len())
}