use std::fs::File;
use std::io::{Error as IOError, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Default)]
pub struct SampleEntry {
    pub name: String,
    pub file_name: String,
    pub channels: u32,
    pub sample_rate: u32,
    pub pos: u32,
}

#[derive(Debug, Default)]
pub struct BankEntry {
    pub name: String,
    pub bank_num: u8,
    pub inst_count: u8,
}

#[derive(Debug, Default)]
pub struct InstEntry {
    pub name: String,
    pub prog: u16,
    pub sdes: u16,
}

#[derive(Debug, Default)]
pub struct SdesEntry {
    pub name: String,

    pub min_pitch: u8,
    pub max_pitch: u8,
    pub base_pitch: u8,
    pub transpose: u8,

    pub vol: u8,
    pub pan: i8,
    pub samp: u8,
}

#[derive(Debug, Default)]
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
                _ => break
            }
        }

        Ok(bank)
    }

    fn read_samples<T: SimpleReader>(&mut self, reader: &mut T) -> Result<(), IOError> {
        let size = reader.read_u32()?;
        let entry_count = size / 22;

        for _ in 0..entry_count {
            reader.seek(SeekFrom::Current(4))?;

            let channels = reader.read_u32()?;
            let sample_rate = reader.read_u32()?;

            reader.seek(SeekFrom::Current(6))?;
            let pos = reader.read_u32()?;

            self.samples.push(SampleEntry {
                channels,
                sample_rate,
                pos,
                ..Default::default()
            })
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
}

pub (crate) trait SimpleReader: Read + Seek {
    fn read_i8(&mut self) -> Result<i8, IOError>;
    fn read_u8(&mut self) -> Result<u8, IOError>;
    fn read_u32(&mut self) -> Result<u32, IOError>;
    fn read_bytes<const N: usize>(&mut self, b: &mut [u8; N]) -> Result<(), IOError>;
    fn read_string(&mut self) -> Result<String, IOError>;
}

impl SimpleReader for File {
    fn read_i8(&mut self) -> Result<i8, IOError> {
        read_i8(self)
    }

    fn read_u8(&mut self) -> Result<u8, IOError> {
        read_u8(self)
    }

    fn read_u32(&mut self) -> Result<u32, IOError> {
        read_u32(self)
    }

    fn read_bytes<const N: usize>(&mut self, b: &mut [u8; N]) -> Result<(), IOError> {
        read_bytes(self, b)
    }

    fn read_string(&mut self) -> Result<String, IOError> {
        read_string(self)
    }
}

fn read_i8<T: Read + Seek>(reader: &mut T)-> Result<i8, IOError> {
    let mut b = [0u8; std::mem::size_of::<i8>()];
    read_bytes(reader, &mut b)?;

    Ok(i8::from_le_bytes(b))
}

fn read_u8<T: Read + Seek>(reader: &mut T)-> Result<u8, IOError> {
    let mut b = [0u8; std::mem::size_of::<u8>()];
    read_bytes(reader, &mut b)?;

    Ok(u8::from_le_bytes(b))
}

fn read_u32<T: Read + Seek>(reader: &mut T)-> Result<u32, IOError> {
    let mut b = [0u8; std::mem::size_of::<u32>()];
    read_bytes(reader, &mut b)?;

    Ok(u32::from_le_bytes(b))
}

fn read_bytes<const N: usize, T: Read + Seek>(reader: &mut T, b: &mut [u8; N]) -> Result<(), IOError> {
    reader.read_exact(b)
}

fn read_string<T: Read + Seek>(reader: &mut T)-> Result<String, IOError> {
    let size = read_u32(reader)?;
    let mut data = vec![0u8; size as usize];

    reader.read_exact(&mut data)?;

    // TODO: Properly map error
    Ok(String::from_utf8(data).unwrap())
}