use anyhow::{anyhow, Context, Result};
use cherryrgb::{
    self, rgb, Brightness, CherryKeyboard, CustomKeyLeds, LightingMode, OwnRGB8, Speed,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct AnimationArgs {
    #[structopt(possible_values = cherryrgb::HELP_LIGHTING_MODE)]
    /// Set LED mode
    mode: LightingMode,

    #[structopt(possible_values = cherryrgb::HELP_SPEED)]
    /// Set speed
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

    #[structopt(long)]
    product_id: Option<u16>,

    // Subcommand
    #[structopt(subcommand)]
    command: CliCommand,

    /// Set brightness
    #[structopt(short, long, default_value = "full", possible_values = cherryrgb::HELP_BRIGHTNESS)]
    brightness: Brightness,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // Search / init usb keyboard
    let devices =
        cherryrgb::find_devices(opt.product_id).context("Failed to find any cherry keyboard")?;

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

    /* Fun begins */
    keyboard
        .fetch_device_state()
        .context("Fetching device state failed")?;

    let loglevel = if opt.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(loglevel).unwrap();

    match opt.command {
        CliCommand::CustomColors(args) => {
            keyboard.reset_custom_colors()?;

            let mut keys = CustomKeyLeds::new();

            for (index, color) in args.colors.into_iter().enumerate() {
                keys.set_led(index, color)?;
            }

            keyboard.set_custom_colors(keys)?;
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

            keyboard
                .set_led_animation(args.mode, opt.brightness, args.speed, color, args.rainbow)
                .context("Failed to set led animation")?;
        }
    }

    Ok(())
}
