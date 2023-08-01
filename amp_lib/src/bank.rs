use crate::SimpleReader;
use serde::{Deserialize, Serialize};
use std::io::{Error as IOError, Read, Seek, SeekFrom, Write};
use std::path::Path;

const VAG_BYTES_PER_BLOCK: usize = 16;
const VAG_SAMPLES_PER_BLOCK: usize = 28;

#[derive(Debug, Default, Serialize)]
pub struct SampleEntry {
    pub name: String,
    pub file_name: String,
    #[serde(skip)] pub channels: u32,
    #[serde(skip)] pub sample_rate: u32,
    #[serde(skip)] pub pos: u32,
}

#[derive(Debug, Default, Serialize)]
pub struct BankEntry {
    pub name: String,
    pub bank_num: u8,
    pub inst_count: u8,
}

#[derive(Debug, Default, Serialize)]
pub struct InstEntry {
    pub name: String,
    pub prog: u16,
    pub sdes: u16,
}

#[derive(Debug, Serialize)]
#[repr(u8)]
pub enum SdesPan {
    Left = 0x0,
    Center = 0x40,
    Right = 0x7F,
}

impl Default for SdesPan {
    fn default() -> Self {
        SdesPan::Center
    }
}

impl From<u8> for SdesPan {
    fn from(num: u8) -> Self {
        match num {
            0x0 => Self::Left,
            0x40 => Self::Center,
            0x7F => Self::Right,
            // Default
            // TODO: Maybe panic?
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SdesEntry {
    pub name: String,

    pub min_pitch: u8,
    pub max_pitch: u8,
    pub base_pitch: u8,
    pub transpose: u8,

    pub vol: u8,
    pub pan: SdesPan,
    pub samp: u8,
}

#[derive(Debug, Default, Serialize)]
pub struct BankFile {
    pub samples: Vec<SampleEntry>,
    pub banks: Vec<BankEntry>,
    pub insts: Vec<InstEntry>,
    pub sdes: Vec<SdesEntry>,
}

impl BankFile {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self, IOError> {
        // TODO: Make this work with generic stream
        let mut bnk_file = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .unwrap();

        let mut magic = [0u8; 4];

        let mut bank = Self::default();

        while let Ok(_) = bnk_file.read_bytes(&mut magic) {
            match &magic {
                b"SAMP" => {
                    bank.read_samples(&mut bnk_file)?;
                },
                b"SANM" => {
                    let strings = bank.read_strings(&mut bnk_file)?;

                    // Update sample names
                    for (i, str) in strings.into_iter().enumerate() {
                        if let Some(sam) = bank.samples.get_mut(i) {
                            sam.name = str;
                        }
                    }
                },
                b"SAFN" => {
                    let strings = bank.read_strings(&mut bnk_file)?;

                    // Update sample file names
                    for (i, str) in strings.into_iter().enumerate() {
                        if let Some(sam) = bank.samples.get_mut(i) {
                            sam.file_name = str;
                        }
                    }
                },
                b"BANK" => {
                    bank.read_banks(&mut bnk_file)?;
                },
                b"BKNM" => {
                    let strings = bank.read_strings(&mut bnk_file)?;

                    // Update bank names
                    for (i, str) in strings.into_iter().enumerate() {
                        if let Some(bnk) = bank.banks.get_mut(i) {
                            bnk.name = str;
                        }
                    }
                },
                b"INST" => {
                    bank.read_insts(&mut bnk_file)?;
                },
                b"INNM" => {
                    let strings = bank.read_strings(&mut bnk_file)?;

                    // Update inst names
                    for (i, str) in strings.into_iter().enumerate() {
                        if let Some(inst) = bank.insts.get_mut(i) {
                            inst.name = str;
                        }
                    }
                },
                b"SDES" => {
                    bank.read_sdes(&mut bnk_file)?;
                },
                b"SDNM" => {
                    let strings = bank.read_strings(&mut bnk_file)?;

                    // Update sdes names
                    for (i, str) in strings.into_iter().enumerate() {
                        if let Some(sdes) = bank.sdes.get_mut(i) {
                            sdes.name = str;
                        }
                    }
                },
                _ => break
            }
        }

        Ok(bank)
    }

    pub fn extract_samples_to_dir<T: AsRef<Path>, S: AsRef<Path>>(&self, sample_file_path: T, output_dir_path: S) -> Result<(), IOError> {
        let mut sample_file = std::fs::OpenOptions::new()
            .read(true)
            .open(sample_file_path)
            .unwrap();

        let output_dir = output_dir_path.as_ref();

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        // Write samples
        for sample in self.samples.iter() {
            let output_path = output_dir.join(format!("{}.wav", sample.name));

            let mut decoder = grim::audio::VAGDecoder::new();
            let mut vag_block = [0u8; VAG_BYTES_PER_BLOCK];
            sample_file.seek(SeekFrom::Start(sample.pos as u64))?;

            let mut sample_stream = Vec::new();

            // Read until 0x07 flag or EOF
            while sample_file.read_exact(&mut vag_block).is_ok() && vag_block[1] != 0x07 {
                // Decode block into samples
                let decoded_samples = decoder.decode_block(&vag_block);
                sample_stream.append(&mut decoded_samples.to_vec());
            }

            // Create wav file
            let wav = grim::audio::WavEncoder::new(sample_stream.as_slice(), sample.channels as u16, sample.sample_rate);
            wav.encode_to_file(output_path).unwrap(); // TODO: Properly handle error
        }

        // Write config file
        let output_json_path = output_dir.join("bank.json");
        let json_file = grim::io::create_new_file(output_json_path).unwrap();
        serde_json::to_writer_pretty(json_file, self).unwrap();

        //ron_file.write(ron_str.as_bytes()).unwrap();

        Ok(())
    }

    fn read_samples<T: SimpleReader>(&mut self, reader: &mut T) -> Result<(), IOError> {
        let size = reader.read_u32()?;
        let entry_count = size / 22;

        for _ in 0..entry_count {
            reader.seek(SeekFrom::Current(4))?; // Entry size, always 18

            let channels = reader.read_u32()?;
            let sample_rate = reader.read_u32()?;

            reader.seek(SeekFrom::Current(6))?;
            let pos = reader.read_u32()?;

            self.samples.push(SampleEntry {
                channels,
                sample_rate,
                pos,
                ..Default::default()
            });
        }

        Ok(())
    }

    fn read_strings<T: SimpleReader>(&mut self, reader: &mut T) -> Result<Vec<String>, IOError> {
        let size = reader.read_u32()?;
        let end_pos = reader.stream_position()? + size as u64;

        reader.seek(SeekFrom::Current(4))?; // Always 1?

        let mut strings = Vec::new();

        while reader.stream_position()? < end_pos {
            let str = reader.read_string()?;
            strings.push(str);
        }

        Ok(strings)
    }

    fn read_banks<T: SimpleReader>(&mut self, reader: &mut T) -> Result<(), IOError> {
        let size = reader.read_u32()?;
        let entry_count = size / 13;

        for _ in 0..entry_count {
            reader.seek(SeekFrom::Current(4))?; // Entry size, always 9
            reader.seek(SeekFrom::Current(4))?; // Unknown

            let bank_num = reader.read_u8()?;
            reader.seek(SeekFrom::Current(2))?; // Unknown

            let inst_count = reader.read_u8()?;
            reader.seek(SeekFrom::Current(1))?; // Unknown

            self.banks.push(BankEntry {
                bank_num,
                inst_count,
                ..Default::default()
            });
        }

        Ok(())
    }

    fn read_insts<T: SimpleReader>(&mut self, reader: &mut T) -> Result<(), IOError> {
        let size = reader.read_u32()?;
        let entry_count = size / 16;

        for _ in 0..entry_count {
            reader.seek(SeekFrom::Current(4))?; // Entry size, always 12
            reader.seek(SeekFrom::Current(4))?; // Unknown, always 1?

            let prog = reader.read_u16()?;
            reader.seek(SeekFrom::Current(4))?; // Unknown

            let sdes = reader.read_u16()?;

            self.insts.push(InstEntry {
                prog,
                sdes,
                ..Default::default()
            });
        }

        Ok(())
    }

    fn read_sdes<T: SimpleReader>(&mut self, reader: &mut T) -> Result<(), IOError> {
        let size = reader.read_u32()?;
        let end_pos = reader.stream_position()? + size as u64;

        while reader.stream_position()? < end_pos {
            reader.seek(SeekFrom::Current(4))?; // Entry size

            let end_bytes = reader.read_u32()?;

            let min_pitch = reader.read_u8()?;
            let max_pitch = reader.read_u8()?;
            let base_pitch = reader.read_u8()?;
            let transpose = reader.read_u8()?;

            reader.seek(SeekFrom::Current(12))?; // Unknown

            let vol = reader.read_u8()?;
            let pan = reader.read_u8()?.into();
            let samp = reader.read_u8()?;

            reader.seek(SeekFrom::Current(3 + end_bytes as i64))?;

            self.sdes.push(SdesEntry {
                min_pitch,
                max_pitch,
                base_pitch,
                transpose,
                vol,
                pan,
                samp,
                ..Default::default()
            });
        }

        Ok(())
    }
}
