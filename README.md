# Cherry RGB Keyboard Library

[![Crates.io](https://img.shields.io/crates/v/cherryrgb.svg)](https://crates.io/crates/cherryrgb)
[![Docs.rs](https://docs.rs/cherryrgb/badge.svg)](https://docs.rs/cherryrgb)
[![GitHub release](https://img.shields.io/github/v/release/skraus-dev/cherryrgb-rs?include_prereleases)](https://github.com/skraus-dev/cherryrgb-rs/releases/latest)
[![CI](https://github.com/skraus-dev/cherryrgb-rs/workflows/CI/badge.svg)](https://github.com/skraus-dev/cherryrgb-rs/actions)

Tested with
* Cherry Keyboard G80-3000N RGB (046a:00dd)

## Library

Find usb keyboard and initialize it

```rs
let mut device_handle = cherryrgb::find_device().unwrap();
cherryrgb::init_device(&mut device_handle).unwrap();
cherryrgb::fetch_device_state(&device_handle).unwrap();
```

Set LED animation

```rs
// Create color: green
let color = cherryrgb::RGB8::new(0, 0xff, 0);
let use_rainbow_colors: bool = false;

cherryrgb::set_led_animation(
    &device_handle,
    cherryrgb::LightingMode::Rain,
    cherryrgb::Brightness::Full,
    cherryrgb::Speed::Slow,
    color,
    use_rainbow_colors,
)
.unwrap();
```

Set custom colors
```rs
// Reset all colors first
cherryrgb::reset_custom_colors(&device_handle).unwrap();

// Create color: green
let color = cherryrgb::RGB8::new(0, 0xff, 0);

// Create keys struct and set key with index 42 to desired color
let mut keys = cherryrgb::CustomKeyLeds::new();
keys.set_led(42, color.into()).unwrap();

// Send packets to keyboard
cherryrgb::set_custom_colors(&device_handle, keys).unwrap();
```

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

## Disclaimer

Use at your own risk.
This project is not affiliated or endorsed by Cherry GmbH. 