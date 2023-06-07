use std::{convert::TryFrom, io::Read, io::Write};

use anyhow::{Context, Result};
use cherryrgb::{
    self, read_color_profile, rgb, Brightness, CustomKeyLeds, LightingMode, OwnRGB8, RpcAnimation,
    Speed,
};
use clap::Parser;
use std::os::unix::net::UnixStream;

mod ncli;
use ncli::{CliCommand, Opt};

struct UnixClient {
    sock: UnixStream,
}

/// UnixClient resembles CherryKeyboard, but connects to service
impl UnixClient {
    pub fn new(path: String) -> Result<Self, anyhow::Error> {
        let sock =
            UnixStream::connect(path.clone()).context(format!("Could not connect to {path}"))?;
        Ok(Self { sock })
    }

    /// Reset custom key colors to default
    pub fn reset_custom_colors(&mut self) -> Result<(), anyhow::Error> {
        self.sock
            .write_all("reset_custom_colors".as_bytes())
            .context("I/O error writing to socket")?;
        Ok(())
    }

    /// Set custom color for each individual key
    pub fn set_custom_colors(&mut self, key_leds: CustomKeyLeds) -> Result<(), anyhow::Error> {
        let json = serde_json::to_string(&key_leds).unwrap();
        self.sock
            .write_all(format!("set_custom_colors={}", json).as_bytes())
            .context("I/O error writing to socket")?;
        Ok(())
    }

    /// Set LED animation from different modes
    pub fn set_led_animation<C: Into<OwnRGB8>>(
        &mut self,
        mode: LightingMode,
        brightness: Brightness,
        speed: Speed,
        color: C,
        rainbow: bool,
    ) -> Result<(), anyhow::Error> {
        let rpc = RpcAnimation {
            mode,
            brightness,
            speed,
            color: Some(color.into()),
            rainbow,
        };
        let json = serde_json::to_string(&rpc).unwrap();
        self.sock
            .write_all(format!("set_led_animation={}", json).as_bytes())
            .context("I/O error writing to socket")?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    let loglevel = if opt.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(loglevel)?;

    let mut keyboard = UnixClient::new(opt.socket_path)?;

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
            let path_str = args
                .file_path
                .to_str()
                .map_or(String::new(), |p| p.to_string());

            let mut f = std::fs::File::open(&args.file_path)
                .context(format!("color profile '{path_str}'"))?;
            let mut json: String = String::new();

            f.read_to_string(&mut json)?;

            let colors_from_file =
                read_color_profile(&json).context("reading colors from color file")?;

            let keys =
                CustomKeyLeds::try_from(colors_from_file).context("assembling custom key leds")?;

            keyboard.set_custom_colors(keys)?;
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
