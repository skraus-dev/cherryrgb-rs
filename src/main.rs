use anyhow::{anyhow, Context, Result};
use cherryrgb::{self, Brightness, LightingMode, Speed, RGB8};
use std::convert::TryFrom;
use structopt::StructOpt;

fn parse_color(src: &str) -> Result<RGB8> {
    let slices = src.split(',').collect::<Vec<&str>>();
    let val = match slices.len() {
        3 => {
            let r = slices[0].parse::<u8>()?;
            let g = slices[1].parse::<u8>()?;
            let b = slices[2].parse::<u8>()?;

            RGB8 { r, g, b }
        }
        _ => {
            return Err(anyhow!("Invalid slice count"));
        }
    };

    Ok(val)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cherryrgb", about = "Test tool for Cherry RGB Keyboard")]
struct Opt {
    /// Set LED mode (range 0-15)
    mode: u8,

    /// Set speed (range 0-4)
    speed: u8,

    /// Set brightness (range 0-4)
    brightness: u8,

    /// Color (e.g 255,255,255)
    #[structopt(short, long, parse(try_from_str = parse_color))]
    color: Option<RGB8>,

    /// Enable rainbow colors
    #[structopt(short, long)]
    rainbow: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // Search / init usb keyboard
    let device_handle = cherryrgb::init_device().context("Failed to init keyboard")?;

    /* Fun begins */
    cherryrgb::start_transaction(&device_handle)?;
    cherryrgb::fetch_device_state(&device_handle).context("Init failed")?;
    cherryrgb::end_transaction(&device_handle)?;

    cherryrgb::start_transaction(&device_handle)?;

    let mode: LightingMode =
        LightingMode::try_from(opt.mode).context("Failed to convert argument: LightingMode")?;
    let speed: Speed = Speed::try_from(opt.speed).context("Failed to convert argument: Speed")?;
    let brightness: Brightness =
        Brightness::try_from(opt.brightness).context("Failed to convert argument: Brightness")?;

    println!(
        "Setting: mode={:?} brightness={:?} speed={:?} color={:?}",
        mode, brightness, speed, opt.color
    );

    let packet_bytes = cherryrgb::led_packet(mode, brightness, Some(speed), opt.color, opt.rainbow);

    println!("Setting mode...");
    cherryrgb::send_payload(&device_handle, &packet_bytes)?;

    // Unknown data
    println!("Setting unknown packet...");
    cherryrgb::send_payload(&device_handle, &[0x00, 0x06, 0x01, 0x18, 0x00, 0x55, 0x01])?;

    cherryrgb::end_transaction(&device_handle)?;

    Ok(())
}
