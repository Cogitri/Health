# Health
<p float="left">
<a href="https://flathub.org/apps/details/dev.Cogitri.Health"><img height='80' alt='Download on Flathub' src='https://flathub.org/api/badge?svg&locale=en'/></a>
<a href="https://circle.gnome.org/"><img height='80' alt='Part of GNOME Circle' src='https://gitlab.gnome.org/Teams/Circle/-/raw/91de93edbb3e75eb0882d56bd466e58b525135d5/assets/button/circle-button-fullcolor.svg'/>
</p>

A health tracking app for the GNOME desktop.

![screenshot](https://gitlab.gnome.org/World/Health/raw/master/docs/screenshot_main.png)

## Hacking on Health

### VSCode

The easiest way to get started with Health is by using the "Flatak" extension for VSCode.
You can setup the dependencies of Health by running the following commands:

```sh
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install org.freedesktop.Sdk.Extension.rust-stable//21.08 org.gnome.Sdk//42
```

Afterwards, simply open the Health Code-Workspace by opening VSCode and using `File -> Open Workspace from File ...`. Install the recommended extensions. Now you can build and run Health with the commands of the Flatpak extension (`Ctrl+Shift+P -> Flatpak: Run`).

## Using CI Snapshots

It's possible to use the flatpak bundles that are built-in merge requests and branches of the Health repository. This allows for quick testing of changes without having to build Health yourself.

To download the flatpak bundle of merge requests, go to the merge request, click on "View exposed artifact" and afterwards on "Get Flatpak bundle here":

![screenshot](https://gitlab.gnome.org/World/Health/raw/master/docs/ci-mr-flatpak-bundle.png)

After downloading the file, you can install it with:

```sh
tar xf repo.tar
flatpak build-bundle repo/ health.flatpak dev.Cogitri.Health.Devel
flatpak install --user health.flatpak
```

As mentioned above, you can use `--system` instead of `--user` in the first and last command to install Health system-wide.
