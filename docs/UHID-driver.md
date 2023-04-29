## New UHID service and correspondig new cli

### Preface

The UHID feature is available **on Linux only**.
The motivation for this new was [this issue](https://github.com/skraus-dev/cherryrgb-rs/issues/22).

### How to build
In order to build everything, you now must use the ``--all`` flag and enable the feature ``uhid``
when building. E.g.:
``cargo build --all --features uhid``
This creates 2 new binaries ``cherryrgb_service`` and ``cherryrgb_ncli``.

The service should run as root and provides the UHID driver as well as socket server, listening on ``/run/cherryrgb.sock`` by default.
The cherryrgb_ncli client works **almost** identical to the cherryrgb_cli, except it communicates with the service
(using the unix socket). Therefore, it does not have the ``--product-id`` option anymore (which has been moved to
the service). Instead, it now has an option ``--socket``, which can be used to specify a non-standard socket-path.
The rest of the options and commands are identical to cherryrgb_cli.

### Service options
The service has some new option to configure it properly:
``--socket`` can specify a path other than the default ``/run/cherryrgb.sock``.
``--socketmode`` can specify a mode (permissions) other that the default ``0644``. (The mode is specified **octal**).
``--socketgroup`` can specify a group other that the default ``root``.

### Installation on a Linux system that uses systemd
For systems that provide systemd, there is a ``.service`` file and a configuration file in the hierarchy below ``service/etc/``.
If you want to use these, install everything like this:

```shell
sudo cp target/*/cherryrgb_service /usr/libexec/
sudo cp service/etc/systemd/system/cherryrgb.service /etc/systemd/system/
sudo cp service/etc/default/cherryrgb /etc/default/
```

Also, perhaps you want to edit ``/etc/default/cherryrgb``.
For example, on my local Fedora installation, administrators are in the group ``wheel``.
Therefore, I have CHERRYRGB_OPTIONS defined like this:
```
CHERRYRGB_OPTIONS="--socketgroup wheel"
```
After adjusting the config to your needs, run:
```shell
sudo systemctl start cherryrgb.service
```
and check for errors with:
```shell
sudo systemctl status cherryrgb.service
```

If the service is running, you should be able to run  ``cherryrgb_ncli`` with the same commands you used with cherryrgb_cli.
If everything worked out ok, enable the service to be started at boot:
```shell
sudo systemctl enable cherryrgb.service
```

Finally, in order to recover if the keyboard is unplugged/plugged, install the supplied udev rule:
```shell
sudo cp udev/99-cherryrgb-service.rules /etc/udev/rules.d/
```
If your systemctl binary is NOT located in /usr/bin you might need to adapt it's path in /etc/udev/rules.d/99-cherryrgb-service.rules
