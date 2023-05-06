mod bnk2wav;

use bnk2wav::*;
use clap::{Parser, Subcommand};

// From Cargo.toml
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) trait SubApp {
    fn process(self) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Parser, Debug)]
#[command(name = PKG_NAME, version = VERSION, about = "Tool for Amplitude PS2", long_about = None)]
struct Options {
    #[command(subcommand)]
    commands: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(name = "bnk2wav", about = "Extract audio samples from .bnk")]
    Bnk2Wav(Bnk2WavApp),
}

#[derive(Debug)]
pub struct AmpTool {
    options: Options,
}

impl AmpTool {
    pub fn new() -> Self {
        Self {
            options: Options::parse()
        }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.options.commands {
            SubCommand::Bnk2Wav(app) => app.process(),
        }
    }
}