use anyhow::{Context, Result};
use cherryrgb::{self, rgb, Brightness, CustomKeyLeds, LightingMode, OwnRGB8, Speed};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct AnimationArgs {
    /// Set LED mode (range 0-15)
    mode: LightingMode,

    /// Set speed (range 0-4)
    speed: Speed,

    /// Color (e.g ff00ff)
    color: Option<OwnRGB8>,

    /// Enable rainbow colors
    #[structopt(short, long)]
    rainbow: bool,
}

#[derive(StructOpt, Debug)]
struct CustomColorOptions {
    colors: Vec<OwnRGB8>,
}

#[derive(StructOpt, Debug)]
enum CliCommand {
    Animation(AnimationArgs),
    CustomColors(CustomColorOptions),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "cherryrgb", about = "Test tool for Cherry RGB Keyboard")]
struct Opt {
    /// Enable debug output
    #[structopt(short, long)]
    debug: bool,

    // Subcommand
    #[structopt(subcommand)]
    command: CliCommand,

    /// Set brightness (range 0-4)
    #[structopt(short, long, default_value = "full")]
    brightness: Brightness,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // Search / init usb keyboard
    let mut device_handle = cherryrgb::find_device().context("Failed to find keyboard")?;
    cherryrgb::init_device(&mut device_handle).context("Failed to init keyboard")?;

    /* Fun begins */
    cherryrgb::fetch_device_state(&device_handle).context("Fetching device state failed")?;

    let loglevel = if opt.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(loglevel).unwrap();

    match opt.command {
        CliCommand::CustomColors(args) => {
            cherryrgb::reset_custom_colors(&device_handle)?;

            let mut keys = CustomKeyLeds::new();

            for (index, color) in args.colors.into_iter().enumerate() {
                keys.set_led(index, color)?;
            }

            cherryrgb::set_custom_colors(&device_handle, keys)?;
        }
        CliCommand::Animation(args) => {
            let color = args
                .color
                .or_else(|| Some(rgb::RGB8::new(255, 255, 255).into()))
                .unwrap();

            log::info!(
                "Setting: mode={:?} brightness={:?} speed={:?} color={:?}",
                args.mode,
                opt.brightness,
                args.speed,
                color
            );

            cherryrgb::set_led_animation(
                &device_handle,
                args.mode,
                opt.brightness,
                args.speed,
                color,
                args.rainbow,
            )
            .context("Failed to set led animation")?;
        }
    }

    Ok(())
}
