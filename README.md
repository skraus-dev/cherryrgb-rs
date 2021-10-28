# Cherry RGB Keyboard Library

[![Crates.io](https://img.shields.io/crates/v/cherryrgb.svg)](https://crates.io/crates/cherryrgb)
[![Docs.rs](https://docs.rs/cherryrgb/badge.svg)](https://docs.rs/cherryrgb)
[![GitHub release](https://img.shields.io/github/v/release/skraus-dev/cherryrgb-rs?include_prereleases)](https://github.com/skraus-dev/cherryrgb-rs/releases/latest)
[![CI](https://github.com/skraus-dev/cherryrgb-rs/workflows/CI/badge.svg)](https://github.com/skraus-dev/cherryrgb-rs/actions)

Tested with
* Cherry Keyboard G80-3000N RGB (046a:00dd)

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

## Disclaimer

Use at your own risk.
This project is not affiliated or endorsed by Cherry GmbH. 