# Cherry RGB Keyboard util

Tested with
* Cherry Keyboard G80-3000N RGB

```
Bus 001 Device 021: ID 046a:00dd Cherry GmbH CHERRY Keyboard
```

## What does it do?

Sets LED modes of keyboard.
WIP.

## How to use (Development)

Requirements:

* cargo

```
$ cargo run -- --help

cherryrgb 0.1.0
Test tool for Cherry RGB Keyboard

USAGE:
    cherryrgb_cli [FLAGS] [OPTIONS] <mode> <speed> <brightness>

FLAGS:
    -h, --help       Prints help information
    -r, --rainbow    Enable rainbow colors
    -V, --version    Prints version information

OPTIONS:
    -c, --color <color>    Color (e.g 255,255,255)

ARGS:
    <mode>          Set LED mode (range 0-15)
    <speed>         Set speed (range 0-4)
    <brightness>    Set brightness (range 0-4)
```

Example

* Sets color to green
* Mode: Heartbeat
* Speed: 0 (Fast)
* Brightness: Full

```
cargo run -- --color 0,255,0 4 0 4
# Or, when using the binary standalone
./cherryrgb_cli --color 0,255,0 4 0 4
```

## Disclaimer

Use at your own risk.
This project is not affiliated or endorsed by Cherry GmbH. 