use cherryrgb::{self, Brightness};
use clap::Parser;

#[path = "../../src/commonargs.rs"]
mod commonargs;
pub use commonargs::{AnimationArgs, CliCommand, ColorProfileFileOptions, CustomColorOptions};

#[derive(Parser, Debug)]
#[command(name = "cherryrgb_ncli", author, version, about = "Client for service-based Cherry RGB Keyboard", long_about = None)]
pub struct Opt {
    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    #[arg(
        name = "socket",
        short,
        long,
        help = "Path of socket to connect.",
        default_value = "/run/cherryrgb.sock"
    )]
    pub socket_path: String,

    // Subcommand
    #[command(subcommand)]
    pub command: CliCommand,

    /// Set brightness
    #[arg(short, long, default_value_t = Brightness::Full, value_enum)]
    pub brightness: Brightness,
}
