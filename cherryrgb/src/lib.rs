//! Library to interact with Cherry RGB keyboards
//!
//! # Usage
//!
//! Find usb keyboard and initialize it
//! ```rust no_run
//! use cherryrgb::{self, CherryKeyboard};
//!
//! // Optionally, filter for product id if you have more than one cherry device.
//! let devices = cherryrgb::find_devices(Some(0x00dd)).unwrap();
//! let (vendor_id, product_id) = devices.first().unwrap().to_owned();
//! let keyboard = CherryKeyboard::new(vendor_id, product_id).unwrap();
//!
//! keyboard.fetch_device_state().unwrap();
//! ```
//!
//! Set LED animation
//! ```rust no_run
//! # let keyboard = cherryrgb::CherryKeyboard::new(0, 0).unwrap();
//! use cherryrgb::rgb::RGB8;
//!
//! // Create color: green
//! let color = RGB8::new(0, 0xff, 0);
//! let use_rainbow_colors: bool = false;
//!
//! keyboard.set_led_animation(
//!     cherryrgb::LightingMode::Rain,
//!     cherryrgb::Brightness::Full,
//!     cherryrgb::Speed::Slow,
//!     color,
//!     use_rainbow_colors,
//! )
//! .unwrap();
//! ```
//!
//! Set custom LED color for individual key(s)
//! ```rust no_run
//! # let keyboard = cherryrgb::CherryKeyboard::new(0, 0).unwrap();
//! use cherryrgb::rgb::RGB8;
//!
//! // Reset all colors first
//! keyboard.reset_custom_colors().unwrap();
//!
//! // Create color: green
//! let color = RGB8::new(0, 0xff, 0);
//!
//! // Create keys struct and set key with index 42 to desired color
//! let mut keys = cherryrgb::CustomKeyLeds::new();
//! keys.set_led(42, color).unwrap();
//!
//! // Send packets to keyboard
//! keyboard.set_custom_colors(keys).unwrap();
//! ```

mod extensions;
mod models;

use anyhow::{anyhow, Context, Result};
use binrw::BinReaderExt;
use models::ProfileKey;
use rgb::RGB8;
use rusb::UsbContext;
use serde_json::{self, Value};
use std::{str::FromStr, time::Duration};

// Re-exports
pub use extensions::{OwnRGB8, ToVec};
pub use hex;
pub use models::{Brightness, CustomKeyLeds, LightingMode, Packet, Payload, Speed};
pub use rgb;
pub use rusb;

// Constants
/// USB Vendor ID - Cherry GmbH
pub const CHERRY_USB_VID: u16 = 0x046a;

const INTERFACE_NUM: u8 = 1;
const INTERRUPT_EP: u8 = 0x82;
static TIMEOUT: Duration = Duration::from_millis(1000);

/// Calculate packet checksum (index 1 in payload)
fn calc_checksum(payload_type: u8, data: &[u8]) -> u16 {
    let sum = data.iter().map(|&i| i as u16).sum::<u16>() + (payload_type as u16);

    sum
}

/// Return true, if supplied product id is not blacklisted
fn is_supported(product_id: u16) -> bool {
    let blacklist: Vec<u16> = vec![
        0xc122, // Cherry KC 1000
    ];
    !blacklist.contains(&product_id)
}

/// Find supported Cherry USB keyboards and return collection of (vendor_id, product_id)
pub fn find_devices(product_id: Option<u16>) -> Result<Vec<(u16, u16)>> {
    let devices = rusb::devices()?;
    // Search usb devices with VENDOR_ID of Cherry GmbH
    // If product_id is provided, filter for it too
    let usb_ids: Vec<(u16, u16)> = devices
        .iter()
        .map(|dev| dev.device_descriptor().unwrap())
        .filter(|desc| desc.vendor_id() == CHERRY_USB_VID)
        .filter(|desc| is_supported(desc.product_id()))
        .filter(|desc| match product_id {
            Some(prod_id) => desc.product_id() == prod_id,
            None => true,
        })
        .map(|desc| (desc.vendor_id(), desc.product_id()))
        .collect();

    if usb_ids.is_empty() {
        return Err(anyhow!("No matching devices found"));
    }

    Ok(usb_ids)
}

/// Reads the given color profile and returns a vector of `ProfileKey`.
/// # Arguments
/// * `color_profile` - Color profile content.
pub fn read_color_profile(color_profile: &str) -> Result<Vec<ProfileKey>> {
    let v: Value = serde_json::from_str(color_profile)?;

    v.as_object().map_or(
        Err(anyhow!(format!("No valid colors found in color profile."))),
        |root| {
            root.iter()
                .map(|(key, value)| {
                    let key_index = key
                        .parse::<usize>()
                        .context(format!("parsing key index {}", key))?;
                    let color = value.as_str().map_or(
                        Err(anyhow!(format!(
                            "Invalid color for key with index {key_index}"
                        ))),
                        |hex| match OwnRGB8::from_str(hex) {
                            Ok(color) => Ok(color),
                            Err(e) => Err(anyhow!(e)).context(format!("parsing hex color '{hex}'")),
                        },
                    )?;
                    Ok(ProfileKey::new(key_index, color))
                })
                .collect()
        },
    )
}

/// Holds a handle to the USB keyboard device
pub struct CherryKeyboard {
    device_handle: rusb::DeviceHandle<rusb::Context>,
}

impl CherryKeyboard {
    /// Init USB device by verifying number of configurations and claiming appropriate interface
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self> {
        let ctx = rusb::Context::new().context("Failed to create libusb context")?;

        let mut device_handle = ctx
            .open_device_with_vid_pid(vendor_id, product_id)
            .context("Keyboard not found")?;

        let device = device_handle.device();
        let device_desc = device
            .device_descriptor()
            .context("Failed to read device descriptor")?;
        let config_desc = device
            .active_config_descriptor()
            .context("Failed to get config descriptor")?;

        log::debug!(
            "* Connected to: Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id()
        );

        assert_eq!(device_desc.num_configurations(), 1);
        assert_eq!(config_desc.num_interfaces(), 2);

        // Skip kernel driver detachment for non-unix platforms
        if cfg!(unix) {
            device_handle
                .set_auto_detach_kernel_driver(true)
                .context("Failed to detach active kernel driver")?;
        }

        device_handle
            .claim_interface(INTERFACE_NUM)
            .context("Failed to claim interface")?;

        Ok(Self { device_handle })
    }

    /// Writes a control packet first, then reads interrupt packet
    fn send_payload(&self, payload: Payload) -> Result<Vec<u8>> {
        let packet = Packet::new(payload);

        // Serialize and pad to 64 bytes
        let mut packet_bytes = packet.clone().to_vec();
        packet_bytes.resize(64, 0x00);

        let mut response = [0u8; 64];
        self.device_handle
            .write_control(
                rusb::request_type(
                    rusb::Direction::Out,
                    rusb::RequestType::Class,
                    rusb::Recipient::Interface,
                ),
                0x09,          // Request - SET_REPORT
                0x0204,        // Value - ReportId: 4, ReportType: Output
                0x0001,        // Index
                &packet_bytes, // Data
                TIMEOUT,
            )
            .context("Control Write failure")?;

        log::debug!(
            ">> CONTROL TRANSFER {:?}\n>> {:?}\n",
            hex::encode(&packet_bytes),
            packet,
        );

        self.device_handle
            .read_interrupt(
                INTERRUPT_EP,  // Endpoint
                &mut response, // read buffer
                TIMEOUT,
            )
            .context("Interrupt read failure")?;

        let detail_info = {
            match std::io::Cursor::new(response).read_ne::<Packet<Payload>>() {
                Ok(pkt) => format!("{:?} Checksum valid: {:?}", pkt, pkt.verify_checksum()),
                Err(e) => format!("Failed to parse, err: {:?}", e),
            }
        };
        log::debug!(
            "<< INTERRUPT TRANSFER {:?}\n<< {}\n",
            hex::encode(response),
            detail_info
        );

        Ok(response.to_vec())
    }

    /// Start RGB setting transaction
    fn start_transaction(&self) -> Result<()> {
        self.send_payload(Payload::TransactionStart)?;

        Ok(())
    }

    /// End RGB setting transaction
    fn end_transaction(&self) -> Result<()> {
        self.send_payload(Payload::TransactionEnd)?;

        Ok(())
    }

    /// Just taken 1:1 from usb capture
    pub fn fetch_device_state(&self) -> Result<()> {
        log::trace!("Fetching device state - START");
        self.start_transaction()?;
        self.send_payload(Payload::Unknown3 { unk: 0x22 })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x38,
            data_offset: 0x00,
        })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x38,
            data_offset: 0x38,
        })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x38,
            data_offset: 0x70,
        })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x38,
            data_offset: 0xA8,
        })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x38,
            data_offset: 0xE0,
        })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x38,
            data_offset: 0x118,
        })?;
        self.send_payload(Payload::Unknown7 {
            data_len: 0x2A,
            data_offset: 0x150,
        })?;
        self.send_payload(Payload::Unknown1B {
            data_len: 0x38,
            data_offset: 0x00,
        })?;
        self.send_payload(Payload::Unknown1B {
            data_len: 0x38,
            data_offset: 0x38,
        })?;
        self.send_payload(Payload::Unknown1B {
            data_len: 0x0E,
            data_offset: 0x70,
        })?;
        self.end_transaction()?;
        log::trace!("Fetching device state - END");
        Ok(())
    }

    /// Set LED animation from different modes
    pub fn set_led_animation<C: Into<OwnRGB8>>(
        &self,
        mode: LightingMode,
        brightness: Brightness,
        speed: Speed,
        color: C,
        rainbow: bool,
    ) -> Result<()> {
        log::trace!("Set LED animation - START");
        self.start_transaction()?;
        // Send main payload
        self.send_payload(Payload::SetAnimation {
            unknown: [0x09, 0x00, 0x00, 0x55, 0x00],
            mode,
            brightness,
            speed,
            pad: 0x0,
            rainbow: if rainbow { 1 } else { 0 },
            color: color.into(),
        })?;
        // Send unknown / ?static? bytes
        self.send_payload(Payload::SetAnimation {
            unknown: [0x01, 0x18, 0x00, 0x55, 0x01],
            // Everything after unknown is nulled
            mode: LightingMode::Wave,
            brightness: Brightness::Off,
            speed: Speed::VeryFast,
            pad: 0x0,
            rainbow: 0x0,
            color: RGB8::new(0, 0, 0).into(),
        })?;

        self.end_transaction()?;
        log::trace!("Set LED animation - END");
        Ok(())
    }

    /// Set custom color for each individual key
    pub fn set_custom_colors(&self, key_leds: CustomKeyLeds) -> Result<()> {
        log::trace!("Set custom colors - START");
        // Set custom led mode
        self.set_led_animation(
            LightingMode::Custom,
            Brightness::Full,
            Speed::Slow,
            OwnRGB8::default(),
            false,
        )?;

        for payload in key_leds.get_payloads()? {
            self.send_payload(payload)?;
        }
        log::trace!("Set custom colors - END");
        Ok(())
    }

    /// Reset custom key colors to default
    pub fn reset_custom_colors(&self) -> Result<()> {
        log::trace!("Reset custom colors - START");
        // Create array of blank / off LEDs
        self.set_custom_colors(CustomKeyLeds::new())?;

        // Payloads, type: 0x5
        self.send_payload(Payload::Unknown5 { unk: 0x01 })?;
        self.send_payload(Payload::Unknown5 { unk: 0x19 })?;
        log::trace!("Reset custom colors - END");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::PayloadType;

    use super::*;
    use binrw::BinReaderExt;
    use rgb::{ComponentSlice, RGB8};
    use std::io::Cursor;

    /// Some captures packets
    fn packets() -> Vec<&'static str> {
        //                             brightness
        //     checksum                mode|speed      color
        //        |                     |  |  |         |
        //       vvv                    v  v  v         v
        vec![
            "04 69 01 06 09 00 00 55 00 00 03 02 00 01 FF", //       00 - wave - regular
            "04 6B 01 06 09 00 00 55 00 00 03 04 00 01 FF", //       01 - wave - slow
            "04 67 01 06 09 00 00 55 00 00 03 00 00 01 FF", //       02 - wave - fast
            "04 68 01 06 09 00 00 55 00 00 03 01 00 01 FF", //       03 - wave - another speed
            "04 69 01 06 09 00 00 55 00 01 03 01 00 01 FF", //       04 - spectrum - fast
            "04 68 01 06 09 00 00 55 00 01 03 00 00 01 FF", //       05 - spectrum - fastest
            "04 6C 01 06 09 00 00 55 00 01 03 04 00 01 FF", //       06 - spectrum - slow
            "04 6B 01 06 09 00 00 55 00 01 03 03 00 01 FF", //       07 - breathing
            "04 6C 01 06 09 00 00 55 00 02 03 03 00 01 FF", //       08 - breathing - slow
            "04 74 01 06 09 00 00 55 00 0A 03 03 00 01 FF", //       09 - Rolling
            "04 76 01 06 09 00 00 55 00 0C 03 03 00 01 FF", //       10 - Curve
            "04 79 01 06 09 00 00 55 00 0F 03 03 00 01 FF", //       11 - Scan
            "04 7C 01 06 09 00 00 55 00 12 03 03 00 01 FF", //       12 - Radiation
            "04 EE 01 06 09 00 00 55 00 12 03 03 00 00 7E 00 F4", // 13 - Radiation
            "04 EF 01 06 09 00 00 55 00 13 03 03 00 00 7E 00 F4", // 14 - Ripples - slow
            "04 EC 01 06 09 00 00 55 00 13 03 00 00 00 7E 00 F4", // 15 - Rippples - fast
            "04 EE 01 06 09 00 00 55 00 15 03 00 00 00 7E 00 F4", // 16 - Single Key
            "04 DC 01 06 09 00 00 55 00 03 03 00 00 00 7E 00 F4", // 17 - Static - Purple
            "04 4D 01 06 09 00 00 55 00 03 03 00 00 00 E0 03 00", // 18 - Static - Red
            "04 52 01 06 09 00 00 55 00 08 03 00 00 00 E0 03 00", // 19 - Custom
            // start / end transaction packets
            "04 01 00 01",
            "04 02 00 02",
            // fetch device info packets
            "04 25 00 03 22 00 00",
            "04 3f 00 07 38 00 00",
            "04 77 00 07 38 38 00",
            "04 af 00 07 38 70 00",
            "04 e7 00 07 38 a8 00",
            "04 1f 01 07 38 e0 00",
            "04 58 00 07 38 18 01",
            "04 82 00 07 2a 50 01",
            "04 53 00 1b 38 00 00",
            "04 8b 00 1b 38 38 00",
            "04 99 00 1b 0e 70 00",
            // Unknown
            "04 43 00 0b 38 00 00",
            "04 7b 00 0b 38 38 00",
            "04 b3 00 0b 38 70 00",
            "04 eb 00 0b 38 a8 00",
            "04 23 01 0b 38 e0 00",
            "04 5c 00 0b 38 18 01",
            "04 86 00 0b 2a 50 01",
        ]
    }

    #[test]
    fn test_checksum() {
        for (index, &pkt_str) in packets().iter().enumerate() {
            let pkt =
                hex::decode(pkt_str.replace(' ', "")).expect("Failed to convert pkt hexstream");

            let mut cursor = Cursor::new(&pkt[1..]);
            let expected_checksum: u16 = cursor.read_ne().expect("Failed to read checksum");
            let payload_type: u8 = cursor.read_ne().expect("Failed to read command");
            let calcd_checksum = calc_checksum(payload_type, &pkt[4..]);

            assert_eq!(
                expected_checksum, calcd_checksum,
                "Failed checksum for pkt {} data={:?}",
                index, pkt_str
            );
        }
    }

    #[test]
    fn serialize_rgb8() {
        #[rustfmt::skip]
        assert_eq!(RGB8 {r: 232,g: 211,b: 75}.as_slice(),&[232, 211, 75]);
        #[rustfmt::skip]
        assert_eq!(RGB8 {r: 232, g: 0, b: 75}.as_slice(), &[232, 0, 75]);
        #[rustfmt::skip]
        assert_eq!(RGB8 { r: 0, g: 0, b: 75 }.as_slice(), &[0, 0, 75]);
    }

    #[test]
    fn serialize_led_animation_payload() {
        let buf: Vec<u8> = Payload::SetAnimation {
            unknown: [0x09, 0x00, 0x00, 0x55, 0x00],
            mode: LightingMode::Vortex,
            brightness: Brightness::Full,
            speed: Speed::VerySlow,
            pad: 0x0,
            rainbow: 0,
            color: OwnRGB8::new(244, 255, 100),
        }
        .to_vec();
        assert_eq!(
            vec![0x09, 0x00, 0x00, 0x55, 0x00, 0x05, 0x04, 0x04, 0x00, 0x00, 0xF4, 0xFF, 0x64],
            buf
        );
        let buf: Vec<u8> = Payload::SetAnimation {
            unknown: [0x09, 0x00, 0x00, 0x55, 0x00],
            mode: LightingMode::Vortex,
            brightness: Brightness::Full,
            speed: Speed::VerySlow,
            pad: 0x0,
            rainbow: 1,
            color: OwnRGB8::new(244, 255, 100),
        }
        .to_vec();
        assert_eq!(
            vec![0x09, 0x00, 0x00, 0x55, 0x00, 0x05, 0x04, 0x04, 0x00, 0x01, 0xF4, 0xFF, 0x64],
            buf
        );
        let buf: Vec<u8> = Payload::SetAnimation {
            unknown: [0x09, 0x00, 0x00, 0x55, 0x00],
            mode: LightingMode::Rolling,
            brightness: Brightness::Full,
            speed: Speed::VerySlow,
            pad: 0x0,
            rainbow: 0,
            color: OwnRGB8::new(244, 255, 100),
        }
        .to_vec();
        assert_eq!(
            vec![0x09, 0x00, 0x00, 0x55, 0x00, 0x0A, 0x04, 0x04, 0x00, 0x00, 0xF4, 0xFF, 0x64],
            buf
        );
        let buf: Vec<u8> = Payload::SetAnimation {
            unknown: [0x09, 0x00, 0x00, 0x55, 0x00],
            mode: LightingMode::Vortex,
            brightness: Brightness::Full,
            speed: Speed::Medium,
            pad: 0x0,
            rainbow: 0,
            color: OwnRGB8::new(244, 255, 100),
        }
        .to_vec();
        assert_eq!(
            vec![0x09, 0x00, 0x00, 0x55, 0x00, 0x05, 0x04, 0x02, 0x00, 0x00, 0xF4, 0xFF, 0x64],
            buf
        );
        let buf: Vec<u8> = Payload::SetAnimation {
            unknown: [0x09, 0x00, 0x00, 0x55, 0x00],
            mode: LightingMode::Vortex,
            brightness: Brightness::Low,
            speed: Speed::Medium,
            pad: 0x0,
            rainbow: 0,
            color: OwnRGB8::new(244, 255, 100),
        }
        .to_vec();
        assert_eq!(
            vec![0x09, 0x00, 0x00, 0x55, 0x00, 0x05, 0x01, 0x02, 0x00, 0x00, 0xF4, 0xFF, 0x64],
            buf
        );
    }

    #[test]
    fn prep_packet() {
        let packet = Packet::new(Payload::TransactionStart).to_vec();
        assert_eq!(packet[..4], vec![0x04, 0x01, 0x00, 0x01]);

        let packet = Packet::new(Payload::SetAnimation {
            unknown: [0x01, 0x18, 0x00, 0x55, 0x01],
            // Everything after unknown is nulled
            mode: LightingMode::Wave,
            brightness: Brightness::Off,
            speed: Speed::VeryFast,
            pad: 0x0,
            rainbow: 0x0,
            color: RGB8::new(0, 0, 0x42).into(),
        })
        .to_vec();

        assert_eq!(
            packet[..17],
            vec![
                0x04, 0xB7, 0x00, 0x06, 0x01, 0x18, 0x00, 0x55, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x42
            ]
        );
    }

    #[test]
    fn unhandled_packet() {
        let packet = b"\x04\xEE\x01\x42\x09\x00\x00\x55\x00\x12\x03\x03\x00\x00\x7E\x00\xF4";
        let mut reader = Cursor::new(packet);
        let deserialized: Packet<Payload> =
            reader.read_ne().expect("Failed reading unhandled packet");

        assert_eq!(deserialized.checksum(), 0x1EE);
        // Unknown payload types get mapped to 0xFF
        assert_eq!(deserialized.payload().payload_type(), 0xFF);
        match deserialized.payload() {
            Payload::Unhandled { data } => {
                assert_eq!(
                    data[..],
                    b"\x09\x00\x00\x55\x00\x12\x03\x03\x00\x00\x7E\x00\xF4"[..]
                );
            }
            _ => {
                assert_eq!(1, 2)
            }
        }
    }

    #[test]
    fn deserialize_color_profile() {
        let color_profile = r#"
            {
                "0": "ff0000",
                "1": "00ff00",
                "2": "0000ff"
            }
        "#;

        let match_this: Vec<ProfileKey> = vec![
            ProfileKey::new(0, OwnRGB8::new(255, 0, 0)),
            ProfileKey::new(1, OwnRGB8::new(0, 255, 0)),
            ProfileKey::new(2, OwnRGB8::new(0, 0, 255)),
        ];

        let profile_keys = read_color_profile(color_profile).expect("Failed reading color profile");
        assert_eq!(match_this, profile_keys);
    }
}
