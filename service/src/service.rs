use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "cherryrgb_service", author, version, about = "Service daemon and UHID driver for Cherry RGB Keyboard", long_about = None)]
pub struct Opt {
    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    /// Must be specified if multiple cherry products are detected
    #[arg(short, long)]
    pub product_id: Option<String>,

    /// Path of listening socket to create
    #[arg(name = "socket", short, long, default_value = "/run/cherryrgb.sock")]
    pub socket_path: PathBuf,

    /// Permissions of the socket (octal)
    #[arg(name = "socketmode", short = 'm', long, default_value = "0664")]
    pub socket_mode: String,

    /// Group of the socket
    #[arg(name = "socketgroup", short = 'g', long, default_value = "root")]
    pub socket_group: String,
}
