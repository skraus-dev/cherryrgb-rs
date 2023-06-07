use cherryrgb::{self, Brightness};
use clap::Parser;

#[path = "commonargs.rs"]
mod commonargs;
pub use commonargs::{AnimationArgs, CliCommand, ColorProfileFileOptions, CustomColorOptions};

#[derive(Parser, Debug)]
#[command(name = "cherryrgb_cli", author, version, about = "Test tool for Cherry RGB Keyboard", long_about = None)]
pub struct Opt {
    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    /// Must be specified if multiple cherry products are detected.
    /// Interpreted as hex, if prefixed with '0x', as dec otherwise
    #[arg(short, long)]
    pub product_id: Option<String>,

    // Subcommand
    #[command(subcommand)]
    pub command: CliCommand,

    /// Set brightness
    #[arg(short, long, default_value_t = Brightness::Full, value_enum)]
    pub brightness: Brightness,
}
