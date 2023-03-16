# Cherry RGB Keyboard Library

[![Crates.io](https://img.shields.io/crates/v/cherryrgb.svg)](https://crates.io/crates/cherryrgb)
[![Docs.rs](https://docs.rs/cherryrgb/badge.svg)](https://docs.rs/cherryrgb)
[![GitHub release](https://img.shields.io/github/v/release/skraus-dev/cherryrgb-rs?include_prereleases)](https://github.com/skraus-dev/cherryrgb-rs/releases/latest)
[![CI](https://github.com/skraus-dev/cherryrgb-rs/workflows/CI/badge.svg)](https://github.com/skraus-dev/cherryrgb-rs/actions)

Tested with
* Cherry Keyboard G80-3000N RGB (046a:00dd)
* Cherry Keyboard MX10.0N       (046a:00df)

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

Set LED animation

* Color: #00ff00 (green)
* Mode: Rain
* Speed: slow
* Brightness: medium

```
./cherryrgb_cli -b medium animation rain slow 00ff00
```

Set custom key colors

* Brightness: full
* Key 0 color: #ff00ff
* Key 1 color: #0000ff

```
./cherryrgb_cli -b full custom-colors ff00ff 0000ff
```

## Build from source

### Dependencies

- Rust (https://www.rust-lang.org/tools/install)

### Clone & Build

```bash
git clone https://github.com/skraus-dev/cherryrgb-rs.git
cd cherryrgb-rs
cargo build
```

Now you can run the binary from `./target/debug/cherryrgb_cli`

## Disclaimer

Use at your own risk.
This project is not affiliated or endorsed by Cherry GmbH. 
