#!/bin/sh

xvfb-run -a -s "-screen 0 1024x768x24" flatpak-builder --user --disable-rofiles-fuse --build-shell=health flatpak_app dev.Cogitri.Health.Devel.json <<END
$@
END
