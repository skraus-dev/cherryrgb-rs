# CherryRGB

## Usage

Find usb keyboard and initialize it

```rust
use cherryrgb::{self, CherryKeyboard};

// Optionally, filter for product id if you have more than one cherry device.
let devices = cherryrgb::find_devices(Some(0x00dd)).unwrap();
let (vendor_id, product_id) = devices.first().unwrap().to_owned();
let keyboard = CherryKeyboard::new(vendor_id, product_id).unwrap();

keyboard.fetch_device_state().unwrap();
```

Set LED animation

```rust
// Create color: green
let color = cherryrgb::RGB8::new(0, 0xff, 0);
let use_rainbow_colors: bool = false;

keyboard.set_led_animation(
    cherryrgb::LightingMode::Rain,
    cherryrgb::Brightness::Full,
    cherryrgb::Speed::Slow,
    color,
    use_rainbow_colors,
)
.unwrap();
```

Set custom colors
```rust
// Reset all colors first
keyboard.reset_custom_colors().unwrap();

// Create color: green
let color = cherryrgb::RGB8::new(0, 0xff, 0);

// Create keys struct and set key with index 42 to desired color
let mut keys = cherryrgb::CustomKeyLeds::new();
keys.set_led(42, color.into()).unwrap();

// Send packets to keyboard
keyboard.set_custom_colors(keys).unwrap();
```
