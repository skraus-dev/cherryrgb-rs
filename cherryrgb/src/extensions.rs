use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, ReadOptions, WriteOptions};
use rgb::RGB8;
use std::{
    io::{Cursor, Read, Seek},
    str::FromStr,
};

/// Shorthand for structs implementing BinWrite to serialize into `Vec<u8>`
pub trait ToVec: BinWrite {
    /// Shorthand for serializing into a Vec
    fn to_vec(self) -> Vec<u8>;
}

impl<T> ToVec for T
where
    <T as BinWrite>::Args: Default,
    T: BinWrite,
{
    fn to_vec(self) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        self.write_to(&mut buf).unwrap();
        buf.into_inner()
    }
}

/// Wrap around RGB8 type, to implement traits on it
#[derive(Clone, Default, Debug, PartialEq)]
pub struct OwnRGB8(RGB8);

impl OwnRGB8 {
    /// Create new instance by providing values for red, green, blue
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(RGB8 { r, g, b })
    }
}

impl From<RGB8> for OwnRGB8 {
    fn from(val: RGB8) -> Self {
        Self(val)
    }
}

impl BinRead for OwnRGB8 {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let rgb = RGB8 {
            r: reader.read_ne()?,
            g: reader.read_ne()?,
            b: reader.read_ne()?,
        };

        Ok(Self(rgb))
    }
}

impl BinWrite for OwnRGB8 {
    type Args = ();

    fn write_options<W: std::io::Write + Seek>(
        &self,
        writer: &mut W,
        _: &WriteOptions,
        _: Self::Args,
    ) -> BinResult<()> {
        writer.write_ne(&self.0.r)?;
        writer.write_ne(&self.0.g)?;
        writer.write_ne(&self.0.b)?;
        Ok(())
    }
}

impl FromStr for OwnRGB8 {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match s.len() {
            6 => {
                let bytes = hex::decode(s).unwrap();
                RGB8 {
                    r: bytes[0],
                    g: bytes[1],
                    b: bytes[2],
                }
                .into()
            }
            _ => {
                return Err("Invalid hex string");
            }
        };

        Ok(val)
    }
}
