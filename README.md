# Cherry RGB Keyboard Library

[![Crates.io](https://img.shields.io/crates/v/cherryrgb.svg)](https://crates.io/crates/cherryrgb)
[![Docs.rs](https://docs.rs/cherryrgb/badge.svg)](https://docs.rs/cherryrgb)
[![GitHub release](https://img.shields.io/github/v/release/skraus-dev/cherryrgb-rs?include_prereleases)](https://github.com/skraus-dev/cherryrgb-rs/releases/latest)
[![CI](https://github.com/skraus-dev/cherryrgb-rs/workflows/New%20CI/badge.svg)](https://github.com/skraus-dev/cherryrgb-rs/actions)

## Compatibility

To see which devices are tested take a look at the [Compatibility Table](./docs/COMPATIBILITY.md).

## Contributing

See [CONTRIBUTING](./docs/CONTRIBUTING.md) Guidelines.

## Features

Done

* Set LED animation
* Set single-/multi-color (rainbow)
* Set LED brightness
* Set LED color per key
* Load color profiles from files

Missing

* Individual keymapping

## Library

Please see [Docs.rs](https://docs.rs/cherryrgb)


## UHID Service

To workaround the issue of ["slow keypresses"](#keyboard-device-is-not-discovered-aka-keyboard-not-found-in-normal-user-context), @felfert implemented a userspace HID driver/service and a corresponding CLI.

Check out [UHID Driver documentation](./docs/UHID-driver.md) on how to install and use it.

## CLI

Documentation for the CLI is now generated in separate files:

* [cherryrgb_cli](docs/cherryrgb_cli.md)

Alternative  CLI and service for Linux

* [cherryrgb_ncli](docs/cherryrgb_ncli.md)
* [cherryrgb_service](docs/cherryrgb_service.md)

## Usage examples and remarks

### Set LED animation

* Color: #00ff00 (green)
* Mode: wave
* Speed: fast
* Brightness: medium

```shell
./cherryrgb_cli --brightness medium animation wave fast 00ff00
```

#### Unofficial animation modes

Please note, that the following modes are unofficial and therefore are not guaranteed to work
properly:

* `radar`
* `vortex`
* `fire`
* `stars`
* `rain`

### Set custom key colors

* Brightness: full
* Key 0 color: #ff00ff
* Key 1 color: #0000ff

```shell
./cherryrgb_cli --brightness full custom-colors ff00ff 0000ff
```

### Color profile file

In addition to specifying custom colors via arguments you can create your custom color profiles in a separate file and pass the path of the file as an argument.

Color profile file structure:

```json
{
    "0": "ff00ff",
    "1": "fffff",
    "45": "00ff00",
    "31": "ff0000"
}
```

A profile file is a JSON file that contains a root object and a key value pair for each key. Both key and value *MUST* be strings. The JSON parser now has ben changed slightly to allow for 2 normally unsupported variations:
* C99-style comments (Starting at `//` until the end of a line).
* A trailing comma (after the last key value pair) is ignored.
Each key is identified by its index. The colors are specified using hexadecimal color codes. The maximum usable
index is 124 (currently hardcoded).

Example:

```shell
./cherryrgb_cli --brightness full color-profile-file {FILE PATH}
```

Example `bash` script, demonstrating the new `--keep-existing-colors` feature:
```shell
#!/bin/bash
cherryrgb_cli color-profile-file examples/static_rainbow.json
for i in 1 2 3 4 5 ; do
    sleep 0.5
    cherryrgb_cli color-profile-file -k examples/white_f12.json
    sleep 0.5
    cherryrgb_cli color-profile-file -k examples/red_f12.json
done
```
**Note:**
Because existing colors cannot be read from the keyboard, they are stored in a local cache
after setting them. Therfore, in order to use this feature, the command `color-profile-file`
has to be invoked at least once before.

## Build from source

### Dependencies

* Rust (<https://www.rust-lang.org/tools/install>)

### Clone & Build

```shell
git clone https://github.com/skraus-dev/cherryrgb-rs.git
cd cherryrgb-rs
cargo build
cargo xtask all
```
For a **complete** build on Linux, you can append `--all-features --all` to the `cargo build` line.

Now you can run the binary from `./target/debug/cherryrgb_cli`

### Generated documentation and shell completion scripts

When running the above command `cargo xtask all`, a directory hierarchy
is generated:
```
target/generated/docs/
                 man/
                 completions\
```
which contain:
- Markup documentation
- Unix man pages. Copy those into your [manpath](https://man7.org/linux/man-pages/man1/manpath.1.html), where
the extension of each file indicates the man secrion (usually a subdirectory named `man1`, `man8` etc.)
- Shell completion scripts for `bash`, `elvish`, `fish`, `powershell` and `zsh`.
To use the completion scripts, you mus copy them to the appropriate location
(which depends both on your shell and system). For example, on Fedora, the bash
completion scripts go into `/etc/bash_completion.d/`. Refer to the documentation
of your shell/system.

## Install through package manager

- AUR: [<code>cherryrgb</code>](https://aur.archlinux.org/packages/cherryrgb)
- Fedora: [<code>cherryrgb</code>](https://copr.fedorainfracloud.org/coprs/felfert/cherryrgb/) [![Copr build status](https://copr.fedorainfracloud.org/coprs/felfert/cherryrgb/package/cherryrgb/status_image/last_build.png)](https://copr.fedorainfracloud.org/coprs/felfert/cherryrgb/package/cherryrgb/)

## Troubleshooting

### Keyboard device is not discovered aka. "Keyboard not found" in normal user context

If the interaction with the keyboard is fine as root-user, you need to configure udev
to adjust the ownership of the device handle, so a regular user can access it.

The repository contains an example udev rule file [`udev/99-cherryrgb.rules`](https://github.com/skraus-dev/cherryrgb-rs/blob/master/udev/99-cherryrgb.rules).

You might want to adjust it to only handle your specific product id (check via `lsusb`).

In the following example we assume your product id is **0x00dd**.

1. (optional) Change `ATTR{idProduct}=="*"` to `ATTR{idProduct}=="00dd"`.

2. Copy the file to the correct location: `cp 99-cherryrgb.rules /etc/udev/rules.d/` (as a privileged user)

3. Finally, reload the udev rules via `udevadm control --reload` and apply them using `udevadm trigger` or by re-plugging your keyboard.

### Keyboard events are processed very slow after setting LEDs

This is a known issue in the keyboard firmware.
It is mentioned here: <https://bbs.archlinux.org/viewtopic.php?id=267365>

- **Proper** way to fix it: **Contact Cherry Support**
- **Workaround**: Comment out the respective line in [`99-cherryrgb.rules`](https://github.com/skraus-dev/cherryrgb-rs/blob/master/udev/99-cherryrgb.rules) and reload/trigger the udev rule.
- See [this](docs/UHID-driver.md) doc for an alternative solution on Linux.

## Disclaimer

Use at your own risk.
This project is not affiliated or endorsed by Cherry GmbH.

## Changelog

### v0.2.7 - 03/06/2023

* deps: Bump binrw to v0.11.2 (thx @felfert once again :))

### v0.2.6 - 18/05/2023

* meta: docs.rs buildfix (by @felfert)

### v0.2.5 - 17/05/2023

* lib: Query rusb for detach support (by @felfert)
* general: Add ppc64le build (by @felfert)
* compatibility report: G80-3000N FL RGB (by @TheBiochemic)
* docs: Add documentation about service usage (by @felfert)
* service: Implement service daemon and new client (uhid) (by @felfert)

### v0.2.4 - 25/04/2023

* lib: Rework fetch_device_state to use GetKeymap and GetKeyIndexes
* lib: Use thiserror, deprecate anyhow
* lib: Re-export strum-crate
* lib: Implement auto kernel-driver handling (detach / attach) (by @felfert)
* README/udev: Add workaround for sluggish keyevents via udev; however it makes special keys not work

### v0.2.3 - 15/04/2023

* cli: Init logger before sending first packets
* cli: Functionality to pass in colors for indexed keys via json file (by @luv4bytes)
* README: Note about reloading udev rules (by @mpldr)
* Compatibility report: MX BOARD 3.0S FL RGB (by @luv4bytes)
* Compatibility report: G80 3000 TKL RGB (by @cewbdex)

### v0.2.2 - 29/03/2023

* fix: Skip kernel driver detaching for non-unix platforms
* Refactor parameter handling and help for enums (by @felfert)
* Filter unsupported Cherry keyboards (by @felfert)
* Improve README with usage and troubleshooting
* Add example udev rules file

### v0.2.1 - 08/08/2021

* Refactor internal API
* Models: Correct data_offset and checksum fields from u8 to u16

### v0.2.0 - 29/07/2021

* API: Improve usability by wrapping device communication inside struct CherryKeyboard

### v0.1.2 - 28/07/2021

* Implement enumerating all connected Cherry GmbH devices

### v0.1.1 - 28/07/2021

* Differentiate between payload and flags/commands
* Rename LightingModes: Radar, Stars
* Fix bug with missing padding
* Add custom LED color setting
* General code cleanup

### v0.1.0 - 24/07/2021

* Initial release
