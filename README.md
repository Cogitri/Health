# Health

A health tracking app for the GNOME desktop.

![screenshot](https://gitlab.gnome.org/Cogitri/gnome-health/raw/master/docs/screenshot.png)

## Building from Source

## Building in Flatpak

You can build Health with the following command:

```sh
flatpak-builder --user --install --force-clean app dev.Cogitri.Health.json
```

You can also pass `--system` instead of `--user` to flatpak-builder to install Health system-wide instead of only for your user. However, that requires root permissions.

Afterwards you can run it with:

```sh
flatpak run dev.Cogitri.Health
```

For development purposes you can also run Health directly via flatpak-builder to avoid time cost of packing the flatpak and then installing it like so:

```sh
flatpak-builder --user --force-clean app dev.Cogitri.Health.json
flatpak-builder --run app dev.Cogitri.Health.json dev.Cogitri.Health
```
