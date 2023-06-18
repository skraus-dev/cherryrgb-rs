# Command-Line Help for `cherryrgb_ncli`

This document contains the help content for the `cherryrgb_ncli` command-line program.

**Command Overview:**

* [`cherryrgb_ncli`↴](#cherryrgb_ncli)
* [`cherryrgb_ncli animation`↴](#cherryrgb_ncli-animation)
* [`cherryrgb_ncli custom-colors`↴](#cherryrgb_ncli-custom-colors)
* [`cherryrgb_ncli color-profile-file`↴](#cherryrgb_ncli-color-profile-file)

## `cherryrgb_ncli`

Client for service-based Cherry RGB Keyboard

**Usage:** `cherryrgb_ncli [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `animation` — Configure RGB keyboard illumination
* `custom-colors` — Configure custom RGB colors
* `color-profile-file` — Configure custom RGB colors from file

###### **Options:**

* `-d`, `--debug` — Enable debug output
* `-s`, `--socket <socket>` — Path of socket to connect

  Default value: `/run/cherryrgb.sock`
* `-b`, `--brightness <BRIGHTNESS>` — Set brightness

  Default value: `full`

  Possible values: `off`, `low`, `medium`, `high`, `full`




## `cherryrgb_ncli animation`

Configure RGB keyboard illumination

**Usage:** `cherryrgb_ncli animation [OPTIONS] <MODE> <SPEED> [COLOR]`

###### **Arguments:**

* `<MODE>` — Set LED mode

  Possible values: `wave`, `spectrum`, `breathing`, `static`, `radar`, `vortex`, `fire`, `stars`, `rain`, `rolling`, `curve`, `wave_mid`, `scan`, `radiation`, `ripples`, `single_key`

* `<SPEED>` — Set speed

  Possible values: `very_fast`, `fast`, `medium`, `slow`, `very_slow`

* `<COLOR>` — Color (e.g ff00ff)

###### **Options:**

* `-r`, `--rainbow` — Enable rainbow colors



## `cherryrgb_ncli custom-colors`

Configure custom RGB colors

**Usage:** `cherryrgb_ncli custom-colors [COLORS]...`

###### **Arguments:**

* `<COLORS>` — One or more RGB color specs (6-digit hex numbers)



## `cherryrgb_ncli color-profile-file`

Configure custom RGB colors from file

**Usage:** `cherryrgb_ncli color-profile-file [OPTIONS] <FILE_PATH>`

###### **Arguments:**

* `<FILE_PATH>` — A json encoded file, specifying key colors

###### **Options:**

* `-k`, `--keep-existing-colors` — If enabled, modifies existing color profile



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
