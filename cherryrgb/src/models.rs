use std::str::FromStr;

use crate::{
    calc_checksum,
    extensions::{OwnRGB8, ToVec},
};
use anyhow::{anyhow, Result};
use binrw::{binrw, until_eof, BinRead, BinWrite, BinWriterExt};

/// Modes support:
/// -> C: Color
/// -> S: Speed
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum LightingMode {
    Wave = 0x00,      // CS
    Spectrum = 0x01,  // S
    Breathing = 0x02, // CS
    Static = 0x03,    // n/A
    Radar = 0x04,     // Unofficial
    Vortex = 0x05,    // Unofficial
    Fire = 0x06,      // Unofficial
    Stars = 0x07,     // Unofficial
    Rain = 0x0B,      // Unofficial (looks like Matrix :D)
    Custom = 0x08,
    Rolling = 0x0A,   // S
    Curve = 0x0C,     // CS
    WaveMid = 0x0E,   // Unoffical
    Scan = 0x0F,      // C
    Radiation = 0x12, // CS
    Ripples = 0x13,   // CS
    SingleKey = 0x15, // CS
}

impl FromStr for LightingMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mode = match s {
            "wave" => LightingMode::Wave,
            "spectrum" => LightingMode::Spectrum,
            "breathing" => LightingMode::Breathing,
            "static" => LightingMode::Static,
            "radar" => LightingMode::Radar,
            "vortex" => LightingMode::Vortex,
            "fire" => LightingMode::Fire,
            "stars" => LightingMode::Stars,
            "rain" => LightingMode::Rain,
            "custom" => LightingMode::Custom,
            "rolling" => LightingMode::Rolling,
            "curve" => LightingMode::Curve,
            "wave_mid" => LightingMode::WaveMid,
            "scan" => LightingMode::Scan,
            "radiation" => LightingMode::Radiation,
            "ripples" => LightingMode::Ripples,
            "single_key" => LightingMode::SingleKey,
            _ => return Err(anyhow!("Invalid mode supplied: {:?}", s)),
        };

        Ok(mode)
    }
}

pub const HELP_LIGHTING_MODE: &'static [&str]  = &[
    "wave", "spectrum", "breathing", "static", "vortex",
    "fire", "stars", "rain", "custom", "rolling", "curve",
    "wave_mid", "scan", "radiation", "ripples", "single_key"
];

/// Probably controlled at OS / driver level
/// Just defined here for completeness' sake
#[binrw]
#[brw(repr = u8)]
#[derive(Eq, PartialEq, Debug)]
pub enum UsbPollingRate {
    Low,    // 125Hz
    Medium, // 250 Hz
    High,   // 500 Hz
    Full,   // 1000 Hz
}

/// LED animation speed
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Speed {
    VeryFast = 0,
    Fast = 1,
    Medium = 2,
    Slow = 3,
    VerySlow = 4,
}

impl FromStr for Speed {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let speed = match s {
            "very_slow" => Speed::VerySlow,
            "slow" => Speed::Slow,
            "medium" => Speed::Medium,
            "fast" => Speed::Fast,
            "very_fast" => Speed::VeryFast,
            _ => return Err(anyhow!("Invalid mode supplied: {:?}", s)),
        };
        Ok(speed)
    }
}

pub const HELP_SPEED: &'static [&str] = &[
    "very_slow", "slow", "medium", "fast", "very_fast"
];

/// LED brightness
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Brightness {
    Off = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Full = 4,
}

impl FromStr for Brightness {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let brightness = match s {
            "off" => Brightness::Off,
            "low" => Brightness::Low,
            "medium" => Brightness::Medium,
            "high" => Brightness::High,
            "full" => Brightness::Full,
            _ => return Err(anyhow!("Invalid mode supplied: {:?}", s)),
        };

        Ok(brightness)
    }
}

pub const HELP_BRIGHTNESS: &'static [&str]  = &[
    "off", "low", "medium", "high", "full"
];

pub trait PayloadType {
    fn payload_type(&self) -> u8;
}

/// Payloads
#[binrw]
#[br(import(payload_type: u8))]
#[derive(Clone, Debug)]
pub enum Payload {
    #[br(pre_assert(payload_type == 0x1))]
    TransactionStart,
    #[br(pre_assert(payload_type == 0x2))]
    TransactionEnd,
    #[br(pre_assert(payload_type == 0x3))]
    Unknown3 { unk: u8 },
    #[br(pre_assert(payload_type == 0x5))]
    Unknown5 { unk: u8 },
    #[br(pre_assert(payload_type == 0x7))]
    Unknown7 { data_len: u8, data_offset: u16 },
    #[br(pre_assert(payload_type == 0x6))]
    SetAnimation {
        unknown: [u8; 5],
        mode: LightingMode,
        brightness: Brightness,
        speed: Speed,
        pad: u8,
        rainbow: u8,
        color: OwnRGB8,
    },
    #[br(pre_assert(payload_type == 0xB))]
    SetCustomLED {
        #[br(temp)]
        #[bw(calc = key_leds_data.len() as u8)]
        data_len: u8,
        data_offset: u16,
        padding: u8,
        #[br(count = data_len)]
        key_leds_data: Vec<u8>,
    },
    #[br(pre_assert(payload_type == 0x1B))]
    Unknown1B { data_len: u8, data_offset: u8 },
    Unhandled {
        #[br(parse_with = until_eof)]
        data: Vec<u8>,
    },
}

impl PayloadType for Payload {
    fn payload_type(&self) -> u8 {
        match self {
            Payload::TransactionStart => 0x1,
            Payload::TransactionEnd => 0x2,
            Payload::Unknown3 { .. } => 0x3,
            Payload::Unknown5 { .. } => 0x5,
            Payload::Unknown7 { .. } => 0x7,
            Payload::SetAnimation { .. } => 0x6,
            Payload::SetCustomLED { .. } => 0xB,
            Payload::Unknown1B { .. } => 0x1B,
            _ => {
                log::error!("Unhandled Payload: {:?}", self);
                0xFF
            }
        }
    }
}

/// Common packet structure
#[binrw]
#[brw(magic = 4u8)]
#[derive(Clone, Debug)]
pub struct Packet<T: BinRead<Args = (u8,)> + BinWrite<Args = ()> + PayloadType> {
    // magic, fixed to 0x04, see `br(magic = ...)`
    checksum: u16,
    #[br(temp)]
    #[bw(calc = inner.payload_type())]
    payload_type: u8,
    #[br(args(payload_type))]
    inner: T,
}

impl<T> Packet<T>
where
    T: BinRead<Args = (u8,)> + BinWrite<Args = ()> + PayloadType + Clone,
{
    pub fn new(inner: T) -> Self {
        let checksum = calc_checksum(inner.payload_type(), &inner.clone().to_vec());

        Self { checksum, inner }
    }

    pub fn checksum(&self) -> u16 {
        self.checksum
    }

    pub fn payload(&self) -> &T {
        &self.inner
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> Result<()> {
        let calculated = calc_checksum(self.inner.payload_type(), &self.inner.clone().to_vec());
        if calculated == self.checksum {
            Ok(())
        } else {
            Err(anyhow!(
                "Invalid checksum, expected: {}, got: {}",
                calculated,
                self.checksum
            ))
        }
    }
}

/// Wrapper around custom LED color for all keys
#[derive(Default, Debug)]
pub struct CustomKeyLeds {
    key_leds: Vec<OwnRGB8>,
}

impl BinWrite for CustomKeyLeds {
    type Args = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        _: &binrw::WriteOptions,
        _: Self::Args,
    ) -> binrw::BinResult<()> {
        for val in &self.key_leds {
            writer.write_ne(val)?;
        }
        Ok(())
    }
}

impl CustomKeyLeds {
    /// (64 byte packet - 4 byte packet header - 4 byte payload header)
    const CHUNK_SIZE: usize = 56;
    const TOTAL_KEYS: usize = 126;

    /// Initialize with inactive colors (000000) for all keys
    pub fn new() -> Self {
        Self {
            key_leds: (0..CustomKeyLeds::TOTAL_KEYS)
                .into_iter()
                .map(|_| OwnRGB8::default())
                .collect(),
        }
    }

    /// Initialize from collection of RGB8 values
    pub fn from_leds<C: Into<OwnRGB8>>(key_leds: Vec<C>) -> Result<Self> {
        if key_leds.len() > CustomKeyLeds::TOTAL_KEYS {
            return Err(anyhow!("Invalid number of key leds"));
        }

        Ok(Self {
            key_leds: key_leds.into_iter().map(|x| x.into()).collect(),
        })
    }

    /// Set color for particular key at provided index
    pub fn set_led<C: Into<OwnRGB8>>(&mut self, key_index: usize, key: C) -> Result<()> {
        if key_index >= self.key_leds.len() {
            return Err(anyhow!("Key index out of bounds"));
        }

        self.key_leds[key_index] = key.into();
        Ok(())
    }

    /// Get array of payloads to be then provided to `send_payload`
    pub fn get_payloads(self) -> Result<Vec<Payload>> {
        let key_data = self.to_vec();

        let result = key_data
            .chunks(CustomKeyLeds::CHUNK_SIZE)
            .into_iter()
            .enumerate()
            .map(|(index, chunk)| {
                let data_offset = index * CustomKeyLeds::CHUNK_SIZE;

                Payload::SetCustomLED {
                    data_offset: data_offset as u16,
                    padding: 0x00,
                    key_leds_data: chunk.to_vec(),
                }
            })
            .collect();

        Ok(result)
    }
}
