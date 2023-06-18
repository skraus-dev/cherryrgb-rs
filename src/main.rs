use std::{convert::TryFrom, fs::File, io::Read};

use anyhow::{anyhow, Context, Result};
use cherryrgb::{self, read_color_profile, rgb, CherryKeyboard, CustomKeyLeds};
use clap::Parser;

mod cli;
use cli::{CliCommand, Opt};
mod common;
mod state;

fn main() -> Result<()> {
    let opt = Opt::parse();

    // Allow the usual hex specifiation (starting with 0x) for the product-id
    let pid = common::get_u16_from_string(opt.product_id);

    // Search / init usb keyboard
    let devices = cherryrgb::find_devices(pid).context("Failed to find any cherry keyboard")?;

    if devices.len() > 1 {
        for (index, &dev) in devices.iter().enumerate() {
            println!("{}) VEN_ID={}, PROD_ID={}", index, dev.0, dev.1);
        }
        return Err(anyhow!(
            "More than one keyboard found, please provide --product-id"
        ));
    }

    let (vendor_id, product_id) = devices.first().unwrap().to_owned();
    let keyboard =
        CherryKeyboard::new(vendor_id, product_id).context("Failed to create keyboard")?;

    let loglevel = if opt.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(loglevel)?;

    /* Fun begins */
    keyboard
        .fetch_device_state()
        .context("Fetching device state failed")?;

    match opt.command {
        CliCommand::CustomColors(args) => {
            keyboard.reset_custom_colors()?;
            let mut keys = CustomKeyLeds::new();

            for (index, color) in args.colors.into_iter().enumerate() {
                keys.set_led(index, color)?;
            }

            keyboard.set_custom_colors(keys)?;
        }
        CliCommand::ColorProfileFile(args) => {
            let mut f = File::open(&args.file_path)
                .context(format!("color profile {:?}", args.file_path))?;
            let mut json: String = String::new();

            f.read_to_string(&mut json)?;
            // Allow // comments
            let re = regex::RegexBuilder::new(r"//.*?$")
                .multi_line(true)
                .build()
                .unwrap();
            json = re.replace_all(&json, "").to_string();
            // Allow trailing comma after last element
            let re = regex::RegexBuilder::new(r",(\s*\})").build().unwrap();
            json = re.replace_all(&json, "$1").to_string();

            log::debug!("{json}");

            let colors_from_file =
                read_color_profile(&json).context("reading colors from color file")?;

            if args.keep_existing {
                let keys = state::load()?
                    .modify_from(colors_from_file)
                    .context("assembling custom key leds")?;
                keyboard.set_custom_colors(keys.clone())?;
                state::save(keys)?;
            } else {
                let keys = CustomKeyLeds::try_from(colors_from_file)
                    .context("assembling custom key leds")?;
                keyboard.set_custom_colors(keys.clone())?;
                state::save(keys)?;
            }
        }
        CliCommand::Animation(args) => {
            let color = args.color.unwrap_or(rgb::RGB8::new(255, 255, 255).into());

            log::info!(
                "Setting: mode={:?} brightness={:?} speed={:?} color={:?}",
                args.mode,
                opt.brightness,
                args.speed,
                color
            );

            keyboard
                .set_led_animation(args.mode, opt.brightness, args.speed, color, args.rainbow)
                .context("Failed to set led animation")?;
        }
    }

    Ok(())
}
