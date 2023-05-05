use std::fs::File;
use std::io::{Error as IOError, Read, Seek};

pub (crate) trait SimpleReader: Read + Seek {
    fn read_i8(&mut self) -> Result<i8, IOError>;
    fn read_u8(&mut self) -> Result<u8, IOError>;
    fn read_u16(&mut self) -> Result<u16, IOError>;
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

    fn read_u16(&mut self) -> Result<u16, IOError> {
        read_u16(self)
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

fn read_u16<T: Read + Seek>(reader: &mut T)-> Result<u16, IOError> {
    let mut b = [0u8; std::mem::size_of::<u16>()];
    read_bytes(reader, &mut b)?;

    Ok(u16::from_le_bytes(b))
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