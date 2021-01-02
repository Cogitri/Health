# Health

A health tracking app for the GNOME desktop.

![screenshot](https://gitlab.gnome.org/Cogitri/gnome-health/raw/master/docs/screenshot_steps.png)

## Building from Source

### With GNOME Builder

Open GNOME Builder, click on `Clone Repository...` and enter the repository URL. Afterwards click on `Clone Project` and you should be all setup. Pressing `F5` or pressing the build button in the topbar should build Health for you.<>

### Building in Flatpak

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

## Using CI Snapshots

It's possible to use the flatpak bundles that are built in merge requests and branches of the Health repository. This allows for quick testing of changes without having to build Health yourself.

To download the flatpak bundle of a merge requests, go to the merge request, click on "View exposed artifact" and afterwards on "Get Flatpak bundle here":

![screenshot](https://gitlab.gnome.org/Cogitri/gnome-health/raw/master/docs/ci-mr-flatpak-bundle.png)

After downloading the file, you can install it with:

```sh
# The GNOME Nightly Sdk is required for development snapshots of Health
flatpak remote-add --user --if-not-exists gnome-nightly https://nightly.gnome.org/gnome-nightly.flatpakrepo

tar xf repo.tar
flatpak build-bundle repo/ health.flatpak dev.Cogitri.Health.Devel
flatpak install --user health.flatpak
```

As mentioned above, you can use `--system` instead of `--user` in the first&last command to install Health system-wide.
