# Health
<p float="left">
<a href="https://flathub.org/apps/details/dev.Cogitri.Health"><img height='80' alt='Download on Flathub' src='https://flathub.org/assets/badges/flathub-badge-en.png'/></a>
<a href="https://circle.gnome.org/"><img height='80' alt='Part of GNOME Circle' src='https://gitlab.gnome.org/Teams/Circle/-/raw/91de93edbb3e75eb0882d56bd466e58b525135d5/assets/button/circle-button-fullcolor.svg'/>
</p>

A health tracking app for the GNOME desktop.

![screenshot](https://gitlab.gnome.org/World/Health/raw/master/docs/screenshot_main.png)

## Hacking on Health

### With GNOME Builder

Open GNOME Builder, click on `Clone Repository...` and enter the repository URL. Afterwards, click on `Clone Project` and you should be all set up. Pressing `F5` or pressing the build button in the top bar should build Health for you.<>

### Building in Flatpak

You can build Health with the following command:

```sh
flatpak-builder --user --install --force-clean app dev.Cogitri.Health.Devel.json
```

You can also pass `--system` instead of `--user` to flatpak-builder to install Health system-wide instead of only for your user. However, that requires root permissions.

Afterwards, you can run it with:

```sh
flatpak run dev.Cogitri.Health.Devel
```

For development purposes you can also run Health directly via flatpak-builder to avoid the time cost of packing the flatpak and then installing it like so:

```sh
flatpak-builder --user --force-clean app dev.Cogitri.Health.json
flatpak-builder --run app dev.Cogitri.Health.json dev.Cogitri.Health
```


### Building manually

If you don't want to use Flatpak, you can build Health manually like so after installing `rust`, `tracker3-devel` and `libadwaita-devel`:

```
meson -Dprofile=development build
ninja -C build
```

Afterwards, you may launch Health by running:

```
ninja -C build run
```


## Using CI Snapshots

It's possible to use the flatpak bundles that are built-in merge requests and branches of the Health repository. This allows for quick testing of changes without having to build Health yourself.

To download the flatpak bundle of merge requests, go to the merge request, click on "View exposed artifact" and afterwards on "Get Flatpak bundle here":

![screenshot](https://gitlab.gnome.org/World/Health/raw/master/docs/ci-mr-flatpak-bundle.png)

After downloading the file, you can install it with:

```sh
# The GNOME Nightly SDK is required for development snapshots of Health
flatpak remote-add --user --if-not-exists gnome-nightly https://nightly.gnome.org/gnome-nightly.flatpakrepo

tar xf repo.tar
flatpak build-bundle repo/ health.flatpak dev.Cogitri.Health.Devel
flatpak install --user health.flatpak
```

As mentioned above, you can use `--system` instead of `--user` in the first&last command to install Health system-wide.
