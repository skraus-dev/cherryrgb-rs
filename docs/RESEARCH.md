# Research

This document describes approaches on how to look into the original Cherry Windows Utility communication.

It can be seen as a loosely coupled list of interesting topics or FAQ related to researching the keyboard.

It's not really targeted at the end-user.

## How to acquire data from Cherry Windows utility easily on non-Windows systems?

```sh
mkdir /tmp/cherry-util && cd /tmp/cherry-util
wget https://download.cherry.de/files/software/CHERRY-UtilSoftSetup-v109.3_signed.exe
7z x CHERRY-UtilSoftSetup-v109.3_signed.exe
7z x app.7z
```

## Interesting files in Cherry Util

Now you can find the files in the following directories:

### `./app/DefaultData/DefaultData.json`

Contains default keymaps.

### `./Skin/XML/DeviceXml/keyboarddevice_*.xml`

Used to render the Cherry Utility UI for a specific keyboard.
Maybe useful to determine actual keycount of each model.

## USB packet capturing

@felfert wrote a nice guide on how to sniff the traffic from a Windows VM via usbmon into tshark on a linux system.

Check it out: <https://github.com/felfert/cherryrgb-rs/blob/reveng/ReverseEngineering.md>