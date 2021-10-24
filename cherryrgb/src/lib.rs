/// CHERRY G80-3000N RGB TKL experiments
/// No warranty or liability for possible damages
/// Use at your own risk!
use std::time::Duration;

use anyhow::{Context, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rusb::{self, UsbContext};

// Re-exports
pub use rgb::{ComponentSlice, RGB8};

// Constants
const USB_VID: u16 = 0x046a;
const USB_PID: u16 = 0x00dd;
const INTERFACE_NUM: u8 = 1;
static TIMEOUT: Duration = Duration::from_millis(1000);

// Commands
#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Command {
    TransactionStart = 0x01,
    TransactionEnd = 0x02,
    Unknown3 = 0x03,
    SetAnimation = 0x06,
    Unknown7 = 0x07,
    Unknown1B = 0x1B,
}

/// Modes support:
/// -> C: Color
/// -> S: Speed
#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum LightingMode {
    Wave = 0x00,      // CS
    Spectrum = 0x01,  // S
    Breathing = 0x02, // CS
    Static = 0x03,    // n/A
    Heartbeat = 0x04, // Unofficial
    Vortex = 0x05,    // Unofficial
    Fire = 0x06,      // Unofficial
    Colors = 0x07,    // Unofficial
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

/// Probably controlled at OS / driver level
/// Just defined here for completeness' sake
#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum UsbPollingRate {
    Low,    // 125Hz
    Medium, // 250 Hz
    High,   // 500 Hz
    Full,   // 1000 Hz
}

/// LED animation speed
#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Speed {
    Fast = 0,
    Faster = 1,
    Medium = 2,
    SlowPlus = 3,
    Slow = 4,
}

/// LED brightness
#[derive(TryFromPrimitive, IntoPrimitive, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Brightness {
    Off = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Full = 4,
}

/// Assemble LED setting payload
///               brightness  rainbow
///                    |         |   COLOR
///                mode|speed    |  R  G  B
///                 |  |  |      |  |  |  |
///                 v  v  v      v  v  v  v
/// "09 00 00 55 00 12 03 03 00 00 7E 00 F4"
pub fn led_payload(
    mode: LightingMode,
    brightness: Brightness,
    speed: Option<Speed>,
    color: Option<RGB8>,
    rainbow: bool,
) -> Vec<u8> {
    let mut payload = vec![0x09, 0x00, 0x00, 0x55, 0x00];

    payload.push(mode.into());
    payload.push(brightness.into());
    payload.push(speed.or(Some(Speed::Slow)).unwrap().into());
    payload.push(0);
    payload.push(rainbow.into());
    if let Some(c) = color {
        payload.extend(c.as_slice());
    }

    payload
}

/// Calculate packet checksum (index 1 in payload)
fn calc_checksum(data: &[u8]) -> u8 {
    let sum = data.iter().map(|&i| i as u32).sum::<u32>();

    (sum & 0xFF) as u8
}

// Prepend magic, checksum, unknown and command to payload
fn prepare_packet(unknown: bool, command: Command, payload: &[u8]) -> Vec<u8> {
    let mut packet = vec![
        0x04,           // Magic
        0x00,           // Checksum (fill in next step)
        unknown.into(), // Unknown flag
        command.into(), // Command
    ];
    // Append payload
    packet.extend(payload);
    // Set checksum
    packet[1] = calc_checksum(&packet[3..]);

    packet
}

/// Writes a control packet first, then reads interrupt packet
pub fn send_payload(
    device: &rusb::DeviceHandle<rusb::Context>,
    unknown: bool,
    command: Command,
    payload: &[u8],
) -> Result<Vec<u8>> {
    // Prepend magic + checksum
    let packet = prepare_packet(unknown, command, payload);

    let mut response = [0u8; 64];
    device
        .write_control(
            0x21,    // RequestType
            0x09,    // Request
            0x0204,  // Value
            0x0001,  // Index
            &packet, // Data
            TIMEOUT,
        )
        .context("Control Write failure")?;

    device
        .read_interrupt(
            0x82,          // Endpoint
            &mut response, // read buffer
            TIMEOUT,
        )
        .context("Interrupt read failure")?;

    Ok(response.to_vec())
}

/// Start RGB setting transaction
pub fn start_transaction(device: &rusb::DeviceHandle<rusb::Context>) -> Result<()> {
    send_payload(device, false, Command::TransactionStart, &[])?;

    Ok(())
}

/// End RGB setting transaction
pub fn end_transaction(device: &rusb::DeviceHandle<rusb::Context>) -> Result<()> {
    send_payload(device, false, Command::TransactionEnd, &[])?;

    Ok(())
}

/// Just taken 1:1 from usb capture
pub fn fetch_device_state(device: &rusb::DeviceHandle<rusb::Context>) -> Result<()> {
    send_payload(device, false, Command::Unknown3, &[0x22])?;
    send_payload(device, false, Command::Unknown7, &[0x38, 0x00])?;
    send_payload(device, false, Command::Unknown7, &[0x38, 0x38])?;
    send_payload(device, false, Command::Unknown7, &[0x38, 0x70])?;
    send_payload(device, false, Command::Unknown7, &[0x38, 0xA8])?;
    send_payload(device, true, Command::Unknown7, &[0x38, 0xE0])?;
    send_payload(device, false, Command::Unknown7, &[0x38, 0x18, 0x01])?;
    send_payload(device, false, Command::Unknown7, &[0x2A, 0x50, 0x01])?;
    send_payload(device, false, Command::Unknown1B, &[0x38, 0x00])?;
    send_payload(device, false, Command::Unknown1B, &[0x38, 0x38])?;
    send_payload(device, false, Command::Unknown1B, &[0x0E, 0x70])?;

    Ok(())
}

pub fn init_device() -> Result<rusb::DeviceHandle<rusb::Context>> {
    // Search / init usb keyboard
    let ctx = rusb::Context::new().context("Failed to create libusb context")?;

    let mut device_handle = ctx
        .open_device_with_vid_pid(USB_VID, USB_PID)
        .context("Keyboard not found")?;

    let device = device_handle.device();
    let device_desc = device
        .device_descriptor()
        .context("Failed to read device descriptor")?;
    let config_desc = device
        .active_config_descriptor()
        .context("Failed to get config descriptor")?;

    println!(
        "* Connected to: Bus {:03} Device {:03} ID {:04x}:{:04x}",
        device.bus_number(),
        device.address(),
        device_desc.vendor_id(),
        device_desc.product_id()
    );

    assert_eq!(device_desc.num_configurations(), 1);
    assert_eq!(config_desc.num_interfaces(), 2);

    let kernel_driver_active = device_handle
        .kernel_driver_active(INTERFACE_NUM)
        .context("kernel_driver_active")?;

    if kernel_driver_active {
        device_handle
            .detach_kernel_driver(INTERFACE_NUM)
            .context("Failed to detach active kernel driver")?;
    }

    device_handle
        .claim_interface(INTERFACE_NUM)
        .context("Failed to claim interface")?;

    Ok(device_handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Some captures packets
    fn packets() -> Vec<&'static str> {
        //                              brightness
        //     checksum                mode|speed      color
        //      |                       |  |  |         |
        //      v                       v  v  v         v
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
                hex::decode(pkt_str.replace(" ", "")).expect("Failed to convert pkt hexstream");

            let expected_checksum = pkt[1];
            let calcd_checksum = calc_checksum(&pkt[3..]);

            assert_eq!(
                expected_checksum, calcd_checksum,
                "Failed checksum for pkt {} data={:?}",
                index, pkt_str
            );
        }
    }

    #[test]
    fn serialize_rgb8() {
        assert_eq!(
            RGB8 {
                r: 232,
                g: 211,
                b: 75
            }
            .as_slice(),
            &[232, 211, 75]
        );
        assert_eq!(
            RGB8 {
                r: 232,
                g: 0,
                b: 75
            }
            .as_slice(),
            &[232, 0, 75]
        );
        assert_eq!(RGB8 { r: 0, g: 0, b: 75 }.as_slice(), &[0, 0, 75]);
    }
}
