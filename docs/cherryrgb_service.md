# Command-Line Help for `cherryrgb_service`

This document contains the help content for the `cherryrgb_service` command-line program.

**Command Overview:**

* [`cherryrgb_service`↴](#cherryrgb_service)

## `cherryrgb_service`

Service daemon and UHID driver for Cherry RGB Keyboard

**Usage:** `cherryrgb_service [OPTIONS]`

###### **Options:**

* `-d`, `--debug` — Enable debug output
* `-p`, `--product-id <PRODUCT_ID>` — Must be specified if multiple cherry products are detected
* `-s`, `--socket <socket>` — Path of listening socket to create

  Default value: `/run/cherryrgb.sock`
* `-m`, `--socketmode <socketmode>` — Permissions of the socket (octal)

  Default value: `0664`
* `-g`, `--socketgroup <socketgroup>` — Group of the socket

  Default value: `root`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
