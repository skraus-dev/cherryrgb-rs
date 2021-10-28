# CherryRGB

## Usage

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
