//! Support for legacy Cherry keyboards using the framebuffer streaming
//! protocol.
//!
//! Known devices: MX BOARD 6.0 RGB (USB PID 0x00B8)
//!
//! Protocol summary (reverse engineered from USB captures of the original
//! Cherry utility for this keyboard):
//!
//! - Vendor defined HID interface (usage page 0xFF01) with interrupt OUT and
//!   interrupt IN endpoints, 64 byte packets, no report IDs
//! - One frame consists of 9 output packets:
//!   * 8 chunk packets: `c1 3d <seq 01-08>` + 15 keys * 4 bytes + 1 pad byte
//!   * 1 tail packet: `c1 21 09` + 7 keys * 4 bytes + zero padding
//!   * 8 * 15 + 7 = 127 key slots in total
//! - Per key 4 bytes: `[brightness][blue][green][red]`, channels 0..=0x3f
//! - Every output packet is acknowledged with a `c1 01 00 ...` input packet
//! - The keyboard additionally emits `e0 23 ...` input packets carrying
//!   HID keycodes of pressed keys (used by the original utility's key
//!   detection, not needed for lighting control)
//! - The firmware does not store frames: the host has to keep streaming
//!   (~25 fps), otherwise the LEDs decay within a second
//! - All lighting effects are rendered on the host

use std::collections::VecDeque;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use hidapi::HidApi;

use crate::extensions::OwnRGB8;
use crate::rgb::RGB8;
use crate::{Brightness, CherryRgbError, LightingMode, Speed};

/// USB product ids of keyboards speaking the legacy framebuffer protocol
const LEGACY_PRODUCT_IDS: &[u16] = &[0x00b8];

/// Number of key slots in one frame
pub const TOTAL_KEYS: usize = 127;
/// Keys per full chunk packet
const CHUNK_KEYS: usize = 15;
/// Keys in the tail packet (8 * 15 + 7 = 127)
const TAIL_KEYS: usize = 7;
/// USB HID packet size
const PACKET_LEN: usize = 64;
/// Maximum value of a color / brightness channel (6 bit)
pub const MAX_CHANNEL: u8 = 0x3f;
/// Target frame duration (~25 fps, matches the original utility)
const FRAME_DURATION: Duration = Duration::from_millis(40);

/// Returns true if the product id belongs to a keyboard speaking the
/// legacy framebuffer protocol
pub fn is_legacy_product_id(product_id: u16) -> bool {
    LEGACY_PRODUCT_IDS.contains(&product_id)
}

/// Map the CLI brightness level to the 6 bit brightness byte
pub fn brightness_byte(brightness: Brightness) -> u8 {
    match brightness {
        Brightness::Off => 0x00,
        Brightness::Low => 0x0f,
        Brightness::Medium => 0x1f,
        Brightness::High => 0x2f,
        Brightness::Full => MAX_CHANNEL,
    }
}

/// Color of a single key slot
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyColor {
    pub brightness: u8,
    pub blue: u8,
    pub green: u8,
    pub red: u8,
}

impl KeyColor {
    /// Build a key color from an RGB value, scaling the 8 bit channels
    /// down to the 6 bit range of the protocol
    pub fn new(brightness: u8, color: OwnRGB8) -> Self {
        Self::from_rgb(brightness, color.rgb())
    }

    /// Build a key color from an RGB8 value, scaling the 8 bit channels
    /// down to the 6 bit range of the protocol
    pub fn from_rgb(brightness: u8, rgb: RGB8) -> Self {
        Self {
            brightness,
            red: rgb.r >> 2,
            green: rgb.g >> 2,
            blue: rgb.b >> 2,
        }
    }

    fn off() -> Self {
        Self::default()
    }

    fn scaled(&self, factor: f32) -> Self {
        let mut key = *self;
        key.brightness = (self.brightness as f32 * factor).round() as u8;
        key
    }
}

fn hid_err(err: hidapi::HidError) -> CherryRgbError {
    CherryRgbError::LegacyError(format!("hidapi: {err}"))
}

/// Assemble one HID output packet (including the leading report id byte,
/// which the OS strips for devices without numbered reports)
fn build_packet(seq: u8, keys: &[KeyColor], tail: bool) -> [u8; PACKET_LEN + 1] {
    let mut packet = [0u8; PACKET_LEN + 1];

    packet[1] = 0xc1;
    packet[2] = if tail { 0x21 } else { 0x3d };
    packet[3] = seq;
    for (index, key) in keys.iter().enumerate() {
        let offset = 4 + index * 4;
        packet[offset] = key.brightness;
        packet[offset + 1] = key.blue;
        packet[offset + 2] = key.green;
        packet[offset + 3] = key.red;
    }

    packet
}

/// Handle to a legacy protocol keyboard
pub struct LegacyKeyboard {
    device: hidapi::HidDevice,
    /// Non-ack packets (key events) collected while draining acks
    pending: Mutex<VecDeque<[u8; PACKET_LEN]>>,
}

impl LegacyKeyboard {
    /// Open the vendor interface of a legacy keyboard
    pub fn new(vendor_id: u16, product_id: u16) -> Result<Self, CherryRgbError> {
        let api = HidApi::new().map_err(hid_err)?;

        let mut path = None;
        for dev in api.device_list() {
            if dev.vendor_id() == vendor_id
                && dev.product_id() == product_id
                && dev.usage_page() == 0xff01
            {
                path = Some(dev.path().to_owned());
                break;
            }
        }

        let path = path.ok_or(CherryRgbError::DeviceNotFoundError)?;
        let device = api.open_path(&path).map_err(hid_err)?;

        Ok(Self {
            device,
            pending: Mutex::new(VecDeque::new()),
        })
    }

    /// Send a single frame (all 9 packets), then drain the input endpoint.
    /// Acknowledgements are discarded, key events read while draining are
    /// queued and can be collected via `read_packet`.
    /// Note: the keyboard forgets the frame within a second unless the
    /// frames keep coming - use one of the streaming methods instead.
    pub fn send_frame(&self, keys: &[KeyColor; TOTAL_KEYS]) -> Result<(), CherryRgbError> {
        for chunk in 0..(TOTAL_KEYS / CHUNK_KEYS) {
            self.send_chunk(
                chunk as u8 + 1,
                &keys[chunk * CHUNK_KEYS..(chunk + 1) * CHUNK_KEYS],
                false,
            )?;
        }
        self.send_chunk(9, &keys[TOTAL_KEYS - TAIL_KEYS..], true)?;
        self.drain_acks();
        Ok(())
    }

    fn send_chunk(&self, seq: u8, keys: &[KeyColor], tail: bool) -> Result<(), CherryRgbError> {
        let packet = build_packet(seq, keys, tail);

        self.device.write(&packet).map_err(hid_err)?;

        Ok(())
    }

    /// Read the next input packet (acknowledgement or key event) with
    /// timeout. Returns None when no packet arrived within the timeout.
    /// Key events read while draining acks are returned first.
    pub fn read_packet(&self, timeout_ms: i32) -> Option<[u8; PACKET_LEN]> {
        if let Some(pkt) = self.pending.lock().unwrap().pop_front() {
            return Some(pkt);
        }
        self.read_raw(timeout_ms)
    }

    fn read_raw(&self, timeout_ms: i32) -> Option<[u8; PACKET_LEN]> {
        let mut buf = [0u8; PACKET_LEN];
        match self.device.read_timeout(&mut buf, timeout_ms) {
            Ok(len) if len > 0 => Some(buf),
            _ => None,
        }
    }

    fn drain_acks(&self) {
        while let Some(pkt) = self.read_raw(5) {
            if pkt[0] == 0xc1 && pkt[1] == 0x01 {
                continue; // acknowledgement
            }
            // key event read while draining: queue it for the caller
            self.pending.lock().unwrap().push_back(pkt);
        }
    }

    /// Write a raw 64-byte output report without any ack handling.
    /// Used for protocol-level commands outside the frame format.
    pub fn write_raw_packet(&self, packet: &[u8; PACKET_LEN]) -> Result<(), CherryRgbError> {
        let mut buf = [0u8; PACKET_LEN + 1];
        buf[1..].copy_from_slice(packet);
        self.device.write(&buf).map_err(hid_err)?;
        Ok(())
    }

    /// Stream a static frame until the process is interrupted (Ctrl+C)
    pub fn stream_static(&self, keys: &[KeyColor; TOTAL_KEYS]) -> Result<(), CherryRgbError> {
        log::info!("Streaming static frame, press Ctrl+C to stop");
        loop {
            self.send_frame(keys)?;
            thread::sleep(FRAME_DURATION);
        }
    }

    /// Stream an animation until the process is interrupted (Ctrl+C).
    /// All effects are rendered on the host, the firmware only displays
    /// whatever the latest frame contained.
    pub fn stream_animation(
        &self,
        mode: LightingMode,
        brightness: Brightness,
        speed: Speed,
        color: OwnRGB8,
        rainbow: bool,
    ) -> Result<(), CherryRgbError> {
        log::info!("Streaming animation {mode:?}, press Ctrl+C to stop");

        let bright = brightness_byte(brightness);
        let speed_step: u32 = match speed {
            Speed::VeryFast => 8,
            Speed::Fast => 4,
            Speed::Medium => 2,
            _ => 1,
        };
        let breathing_period: u32 = match speed {
            Speed::VeryFast => 25,
            Speed::Fast => 40,
            Speed::Medium => 60,
            Speed::Slow => 90,
            Speed::VerySlow => 120,
        };

        let mut tick: u32 = 0;
        let rgb = color.rgb();
        loop {
            let frame = match mode {
                LightingMode::Static => solid_frame(bright, rgb),
                LightingMode::Spectrum => spectrum_frame(bright, tick, 2 * speed_step),
                LightingMode::Wave => wave_frame(bright, rgb, rainbow, tick, speed_step),
                LightingMode::Breathing => breathing_frame(bright, rgb, tick, breathing_period),
                _ => {
                    return Err(CherryRgbError::LegacyError(format!(
                        "animation mode {mode:?} is not supported on legacy keyboards (supported: static, spectrum, wave, breathing)"
                    )));
                }
            };

            self.send_frame(&frame)?;
            thread::sleep(FRAME_DURATION);
            tick = tick.wrapping_add(1);
        }
    }
}

fn solid_frame(bright: u8, rgb: RGB8) -> [KeyColor; TOTAL_KEYS] {
    let key = KeyColor::from_rgb(bright, rgb);
    [key; TOTAL_KEYS]
}

/// All keys share one hue which rotates over time
fn spectrum_frame(bright: u8, tick: u32, hue_step: u32) -> [KeyColor; TOTAL_KEYS] {
    let hue = (tick * hue_step) % 360;
    let key = hsv_to_key(bright, hue, 1.0, 1.0);
    [key; TOTAL_KEYS]
}

/// Rainbow: hue sweep along the key indices, else brightness wave of the
/// given color traveling along the keyboard
fn wave_frame(
    bright: u8,
    rgb: RGB8,
    rainbow: bool,
    tick: u32,
    speed_step: u32,
) -> [KeyColor; TOTAL_KEYS] {
    let mut frame = [KeyColor::off(); TOTAL_KEYS];
    for (index, key) in frame.iter_mut().enumerate() {
        if rainbow {
            let hue = (index as u32 * 3 + tick * speed_step) % 360;
            *key = hsv_to_key(bright, hue, 1.0, 1.0);
        } else {
            let phase = index as f32 / 12.0 + tick as f32 * speed_step as f32 / 6.0;
            let factor = 0.25 + 0.75 * (0.5 + 0.5 * (phase * std::f32::consts::PI).sin());
            *key = KeyColor::from_rgb(bright, rgb).scaled(factor);
        }
    }
    frame
}

fn breathing_frame(bright: u8, rgb: RGB8, tick: u32, period: u32) -> [KeyColor; TOTAL_KEYS] {
    let phase = (tick % period) as f32 / period as f32;
    let factor = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    let key = KeyColor::from_rgb(bright, rgb).scaled(factor);
    [key; TOTAL_KEYS]
}

fn hsv_to_key(bright: u8, hue: u32, sat: f32, val: f32) -> KeyColor {
    let hue = (hue % 360) as f32;
    let c = val * sat;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = val - c;
    let (r, g, b) = match (hue / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let scale = MAX_CHANNEL as f32;
    KeyColor {
        brightness: bright,
        red: ((r + m) * scale).round() as u8,
        green: ((g + m) * scale).round() as u8,
        blue: ((b + m) * scale).round() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_packet_layout() {
        let keys = [KeyColor {
            brightness: 0x3f,
            red: 0x11,
            green: 0x22,
            blue: 0x33,
        }; TOTAL_KEYS];

        let pkt = build_packet(1, &keys[..CHUNK_KEYS], false);
        assert_eq!(pkt.len(), PACKET_LEN + 1);
        assert_eq!(
            &pkt[..9],
            &[0, 0xc1, 0x3d, 0x01, 0x3f, 0x33, 0x22, 0x11, 0x3f]
        );
        // pad byte after 15 keys
        assert_eq!(pkt[4 + CHUNK_KEYS * 4], 0);
    }

    #[test]
    fn tail_packet_layout() {
        let keys = [KeyColor {
            brightness: 0x3f,
            red: 0x11,
            green: 0x22,
            blue: 0x33,
        }; TOTAL_KEYS];

        let pkt = build_packet(9, &keys[TOTAL_KEYS - TAIL_KEYS..], true);
        assert_eq!(pkt[2], 0x21);
        assert_eq!(pkt[3], 9);
        // 7 keys, then zero padding
        assert_eq!(pkt[4 + TAIL_KEYS * 4], 0);
        assert!(pkt[4 + TAIL_KEYS * 4..].iter().all(|&b| b == 0));
    }

    #[test]
    fn legacy_pid_detection() {
        assert!(is_legacy_product_id(0x00b8));
        assert!(!is_legacy_product_id(0x00df));
    }

    #[test]
    fn color_scaling() {
        let key = KeyColor::new(0x3f, OwnRGB8::new(0xff, 0x80, 0x01));
        assert_eq!(key.red, 0x3f);
        assert_eq!(key.green, 0x20);
        assert_eq!(key.blue, 0x00);
    }
}
