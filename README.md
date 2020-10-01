# GNOME Health

A health tracking app for the GNOME desktop.

![screenshot](https://gitlab.gnome.org/Cogitri/gnome-health/raw/master/docs/screenshot.png)

## Building from Source

## Building in Flatpak

You can build Health with the following command:

```sh
flatpak-builder --install --force-clean app org.gnome.Health.json
```

Afterwards you can run it with:

```sh
flatpak run org.gnome.Health
```


## Building Manually

### Dependencies

First off, you need the following dependencies installed:

* gtk+3.0 >= 3.24
* libgee-0.8
* libhandy >= 1.0
* sqlite >= 3.24
* Vala

Afterwards you can build gnome-health like so:

```sh
meson build
meson compile -C build
meson test -C build
meson install -C build
```
