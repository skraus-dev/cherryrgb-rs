use std::path::PathBuf;

use cherryrgb::{self, LightingMode, OwnRGB8, Speed};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct AnimationArgs {
    /// Set LED mode
    #[arg(value_enum)]
    pub mode: LightingMode,

    /// Set speed
    #[arg(value_enum)]
    pub speed: Speed,

    /// Color (e.g ff00ff)
    pub color: Option<OwnRGB8>,

    /// Enable rainbow colors
    #[arg(short, long)]
    pub rainbow: bool,
}

#[derive(Parser, Debug)]
pub struct CustomColorOptions {
    /// One or more RGB color specs (6-digit hex numbers)
    pub colors: Vec<OwnRGB8>,
}

#[derive(Parser, Debug)]
pub struct ColorProfileFileOptions {
    /// If enabled, modifies existing color profile
    #[arg(short, long = "keep-existing-colors")]
    pub keep_existing: bool,

    /// A json encoded file, specifying key colors
    pub file_path: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Configure RGB keyboard illumination
    Animation(AnimationArgs),
    /// Configure custom RGB colors
    CustomColors(CustomColorOptions),
    /// Configure custom RGB colors from file
    ColorProfileFile(ColorProfileFileOptions),
}
