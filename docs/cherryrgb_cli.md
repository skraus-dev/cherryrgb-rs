# Command-Line Help for `cherryrgb_cli`

This document contains the help content for the `cherryrgb_cli` command-line program.

**Command Overview:**

* [`cherryrgb_cli`↴](#cherryrgb_cli)
* [`cherryrgb_cli animation`↴](#cherryrgb_cli-animation)
* [`cherryrgb_cli custom-colors`↴](#cherryrgb_cli-custom-colors)
* [`cherryrgb_cli color-profile-file`↴](#cherryrgb_cli-color-profile-file)

## `cherryrgb_cli`

Test tool for Cherry RGB Keyboard

**Usage:** `cherryrgb_cli [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `animation` — Configure RGB keyboard illumination
* `custom-colors` — Configure custom RGB colors
* `color-profile-file` — Configure custom RGB colors from file

###### **Options:**

* `-d`, `--debug` — Enable debug output
* `-p`, `--product-id <PRODUCT_ID>` — Must be specified if multiple cherry products are detected. Interpreted as hex, if prefixed with '0x', as dec otherwise
* `-b`, `--brightness <BRIGHTNESS>` — Set brightness

  Default value: `full`

  Possible values: `off`, `low`, `medium`, `high`, `full`




## `cherryrgb_cli animation`

Configure RGB keyboard illumination

**Usage:** `cherryrgb_cli animation [OPTIONS] <MODE> <SPEED> [COLOR]`

###### **Arguments:**

* `<MODE>` — Set LED mode

  Possible values: `wave`, `spectrum`, `breathing`, `static`, `radar`, `vortex`, `fire`, `stars`, `rain`, `rolling`, `curve`, `wave_mid`, `scan`, `radiation`, `ripples`, `single_key`

* `<SPEED>` — Set speed

  Possible values: `very_fast`, `fast`, `medium`, `slow`, `very_slow`

* `<COLOR>` — Color (e.g ff00ff)

###### **Options:**

* `-r`, `--rainbow` — Enable rainbow colors



## `cherryrgb_cli custom-colors`

Configure custom RGB colors

**Usage:** `cherryrgb_cli custom-colors [COLORS]...`

###### **Arguments:**

* `<COLORS>` — One or more RGB color specs (6-digit hex numbers)



## `cherryrgb_cli color-profile-file`

Configure custom RGB colors from file

**Usage:** `cherryrgb_cli color-profile-file [OPTIONS] <FILE_PATH>`

###### **Arguments:**

* `<FILE_PATH>` — A json encoded file, specifying key colors

###### **Options:**

* `-k`, `--keep-existing-colors` — If enabled, modifies existing color profile



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
