use anstream::eprintln;
use owo_colors::OwoColorize as _;

use cargo_toml::{Manifest, Package};

use std::{
    boxed::Box,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use clap::{Command, CommandFactory};
use clap_complete::{
    generate_to,
    Shell::{Bash, Elvish, Fish, PowerShell, Zsh},
};

#[path = "../../src/cli.rs"]
mod cli;
#[path = "../../ncli/src/ncli.rs"]
#[allow(clippy::duplicate_mod)]
mod ncli;
#[path = "../../service/src/service.rs"]
mod service;

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Ok(val) = env::var("CARGO_TERM_COLOR") {
        match val.as_str() {
            "never" => anstream::ColorChoice::Never.write_global(),
            "always" => anstream::ColorChoice::Always.write_global(),
            "auto" => anstream::ColorChoice::Auto.write_global(),
            &_ => (),
        }
    }

    if let Err(e) = try_main() {
        eprintln!("{}", e.bright_red().bold());
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    let cmds = all_commands()?;
    match task.as_deref() {
        Some("completions") => gencompletions(cmds)?,
        Some("manpages") => genmanpages(cmds)?,
        Some("markdown") => genmarkdown(cmds)?,
        Some("all") => genall(cmds)?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

completions      generates shell completion scripts
manpages         generate manpages
markdown         generate markdown
all              generate all of the above
"
    )
}

fn print_generated(path: &Path) {
    let rel = path.strip_prefix(project_root()).unwrap().display();
    eprintln!("   {} {}", "Generated".bright_green().bold(), rel);
}

fn gen_for_all_shells(cmd: &mut Command, dir: &Path) -> Result<(), DynError> {
    for shell in [Bash, Elvish, Fish, PowerShell, Zsh] {
        let path = generate_to(shell, cmd, cmd.get_name().to_string(), dir)?;
        print_generated(&path);
    }
    Ok(())
}

fn gencompletions(cmds: Vec<Command>) -> Result<(), DynError> {
    let dir = dist_dir().join("completions");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    for mut cmd in cmds {
        gen_for_all_shells(&mut cmd, &dir)?;
    }
    Ok(())
}

fn genmarkdown(cmds: Vec<Command>) -> Result<(), DynError> {
    let dir = dist_dir().join("docs");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    for cmd in cmds {
        let buffer = clap_markdown::help_markdown_command(&cmd);
        let path = dir.join(cmd.get_name().to_string() + ".md");
        std::fs::write(path.clone(), buffer)?;
        print_generated(&path);
    }
    Ok(())
}

fn genmanpages(cmds: Vec<Command>) -> Result<(), DynError> {
    fn generate(cmd: &Command, dir: &Path, section: &str) -> Result<(), DynError> {
        // `get_display_name()` is `Some` for all instances, except the root.
        let name = cmd.get_display_name().unwrap_or_else(|| cmd.get_name());
        let path = dir.join(format!("{name}.{section}"));
        let mut out: Vec<u8> = Default::default();
        clap_mangen::Man::new(cmd.clone())
            .section(section)
            .render(&mut out)?;
        std::fs::write(path.clone(), out)?;
        print_generated(&path);
        if !name.contains("help") {
            for sub in cmd.get_subcommands() {
                generate(sub, dir, section)?;
            }
        }

        Ok(())
    }

    let mandir = dist_dir().join("man");
    let _ = fs::remove_dir_all(&mandir);
    fs::create_dir_all(&mandir)?;
    for mut cmd in cmds {
        cmd.build();
        let section = cmd.get_next_help_heading().unwrap_or("1");
        generate(&cmd, &mandir, section)?;
    }
    Ok(())
}

fn genall(cmds: Vec<Command>) -> Result<(), DynError> {
    gencompletions(cmds.clone())?;
    genmanpages(cmds.clone())?;
    genmarkdown(cmds)?;
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn dist_dir() -> PathBuf {
    project_root().join("target/generated")
}

fn read_manifests() -> Result<HashMap<String, Package>, DynError> {
    let mut ret = HashMap::new();
    let tlm = Manifest::from_path(project_root().join("Cargo.toml"))?;
    let name = tlm.package.clone().unwrap().name;
    ret.insert(name, tlm.package.unwrap());
    for member in tlm.workspace.unwrap().members {
        let m = Manifest::from_path(project_root().join(member).join("Cargo.toml"))?;
        ret.insert(m.package.clone().unwrap().name, m.package.unwrap());
    }
    //eprintln!("{}", ::serde_json::to_string_pretty(&ret).unwrap());
    Ok(ret)
}

/* TODO: Solve static lifetime issues
fn fix_cmd(pkgs: HashMap<String, Package>, cmd: Command) -> Result<Command, DynError> {
    let pkg = pkgs.get(cmd.get_name()).unwrap();

    Ok(cmd
        .version("1.2.3" /*pkg.version.get().unwrap().as_str()*/)
        .about("whatever" /*pkg.description.unwrap().get().unwrap().as_str()*/)
        .author("myself /*pkg.authors.get().unwrap().get(0).unwrap().as_str()*/
)
    )
}
*/

fn all_commands() -> Result<Vec<Command>, DynError> {
    // We use this in the future to fix version and description
    let _pkgs = read_manifests()?;
    let ret = vec![
        // (Mis-)using next_help_heading to convey the man section to genmanpages()
        cli::Opt::command_for_update()
            .name("cherryrgb_cli")
            .next_help_heading("1"),
        ncli::Opt::command_for_update()
            .name("cherryrgb_ncli")
            .next_help_heading("1"),
        service::Opt::command_for_update()
            .name("cherryrgb_service")
            .next_help_heading("8"),
    ];
    // fix_cmd(pkgs, cli::Opt::command_for_update().name("cherryrgb_cli"));
    Ok(ret)
}
