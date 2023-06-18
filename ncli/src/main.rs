use std::path::PathBuf;
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

#[path = "../../src/state.rs"]
mod state;

struct UnixClient {
    sock: UnixStream,
}

/// UnixClient resembles CherryKeyboard, but connects to service
impl UnixClient {
    const ERR_WRITE: &str = "I/O error writing to socket";

    pub fn new(path: PathBuf) -> Result<Self, anyhow::Error> {
        let sock = UnixStream::connect(path.as_path())
            .context(format!("Could not connect to {path:?}"))?;
        Ok(Self { sock })
    }

    /// Reset custom key colors to default
    pub fn reset_custom_colors(&mut self) -> Result<(), anyhow::Error> {
        writeln!(self.sock, "reset_custom_colors").context(Self::ERR_WRITE)?;
        Ok(())
    }

    /// Set custom color for each individual key
    pub fn set_custom_colors(&mut self, key_leds: CustomKeyLeds) -> Result<(), anyhow::Error> {
        let json = serde_json::to_string(&key_leds).unwrap();
        writeln!(self.sock, "set_custom_colors={}", json).context(Self::ERR_WRITE)?;
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
        writeln!(self.sock, "set_led_animation={}", json).context(Self::ERR_WRITE)?;
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
            let mut f = std::fs::File::open(&args.file_path)
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
