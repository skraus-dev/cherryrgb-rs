# Cherry RGB Keyboard Library

[![Crates.io](https://img.shields.io/crates/v/cherryrgb.svg)](https://crates.io/crates/cherryrgb)
[![Docs.rs](https://docs.rs/cherryrgb/badge.svg)](https://docs.rs/cherryrgb)
[![GitHub release](https://img.shields.io/github/v/release/skraus-dev/cherryrgb-rs?include_prereleases)](https://github.com/skraus-dev/cherryrgb-rs/releases/latest)
[![CI](https://github.com/skraus-dev/cherryrgb-rs/workflows/CI/badge.svg)](https://github.com/skraus-dev/cherryrgb-rs/actions)

## Compatibility

To see which devices are tested take a look at the [Compatibility Table](COMPATIBILITY.md).

## Features

Done

* Set LED animation
* Set single-/multi-color (rainbow)
* Set LED brightness
* Set LED color per key

Missing

* Individual keymapping

## Library

Please see [Docs.rs](https://docs.rs/cherryrgb)

## CLI

Get usage help

```shell
# Top level
./cherryrgb_cli --help

# For each command
./cherryrgb_cli animation --help
./cherryrgb_cli custom-colors --help
```

Set LED animation

* Color: #00ff00 (green)
* Mode: Rain
* Speed: slow
* Brightness: medium

```shell
./cherryrgb_cli --brightness medium animation rain slow 00ff00
```

Set custom key colors

* Brightness: full
* Key 0 color: #ff00ff
* Key 1 color: #0000ff

```shell
./cherryrgb_cli --brightness full custom-colors ff00ff 0000ff
```

## Build from source

### Dependencies

* Rust (<https://www.rust-lang.org/tools/install>)

### Clone & Build

```shell
git clone https://github.com/skraus-dev/cherryrgb-rs.git
cd cherryrgb-rs
cargo build
```

Now you can run the binary from `./target/debug/cherryrgb_cli`

## Troubleshooting

### Keyboard device is not discovered aka. "Keyboard not found" in normal user context

If the interaction with the keyboard is fine as root-user, you need to configure udev
to adjust the ownership of the device handle, so a regular user can access it.

The repository contains an example udev rule file `udev/99-cherryrgb.rules`.

You might want to adjust it to only handle your specific product id (check via `lsusb`).

In the following example we assume your product id is **0x00dd**.

1. (optional) Change `ATTR{idProduct}=="*"` to `ATTR{idProduct}=="00dd"`.

2. Copy the file to the correct location: `cp 99-cherryrgb.rules /etc/udev/rules.d/` (as a privileged user)

3. Finally, reload the udev rules via `udevadm control --reload` and apply them using `udevadm trigger` or by re-plugging your keyboard.

## Disclaimer

Use at your own risk.
This project is not affiliated or endorsed by Cherry GmbH.

## Changelog

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
