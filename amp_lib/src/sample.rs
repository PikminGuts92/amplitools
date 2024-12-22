use crate::SimpleReader;
use serde::{Deserialize, Serialize};
use std::io::{Error as IOError, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Default, Serialize)]
pub struct Sample {
    pub offset: usize,
    pub blocks: Vec<[u8; 16]>,
}

#[derive(Debug, Default, Serialize)]
pub struct SampleFile {
    pub samples: Vec<Sample>,
}

impl SampleFile {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self, IOError> {
        // TODO: Make this work with generic stream
        let mut nse_file = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .unwrap();

        let mut vag_block = [0u8; 16];
        let mut blocks = Vec::new();

        let mut start_pos = 0usize;
        let mut samples = Vec::new();

        while nse_file.read_exact(&mut vag_block).is_ok() {
            if vag_block[1] != 0x07 {
                blocks.push(vag_block.to_owned());
                continue;
            }

            // Vag block terminator reached
            samples.push(Sample {
                offset: start_pos,
                blocks: {
                    let blocks_owned = blocks;
                    blocks = Vec::new();

                    blocks_owned
                }
            });

            start_pos = nse_file.stream_position()? as usize;
        }

        if !blocks.is_empty() {
            // End of file reached before terminator
            // (shouldn't occur but just in case)
            samples.push(Sample {
                offset: start_pos,
                blocks
            });
        }

        Ok(Self {
            samples
        })
    }

    pub fn extract_samples_to_dir<T: AsRef<Path>>(&self, output_dir_path: T) -> Result<(), IOError> {
        let output_dir = output_dir_path.as_ref();

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        // Write samples
        for (i, sample) in self.samples.iter().enumerate() {
            let output_path = output_dir.join(format!("sample_{}.wav", i));

            let mut decoder = grim::audio::VAGDecoder::new();
            let mut sample_stream = Vec::new();

            for block in sample.blocks.iter() {
                let decoded_samples = decoder.decode_block(block);
                sample_stream.append(&mut decoded_samples.to_vec());
            }

            // Create wav file
            let wav = grim::audio::WavEncoder::new(sample_stream.as_slice(), 1, 22_050);
            wav.encode_to_file(output_path)?;
        }

        Ok(())
    }
}