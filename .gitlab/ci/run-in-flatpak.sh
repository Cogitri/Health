#!/bin/sh

xvfb-run -a -s "-screen 0 1024x768x24" flatpak-builder --user --disable-rofiles-fuse --build-shell=health flatpak_app dev.Cogitri.Health.Devel.json <<END
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --error-exitcode=1"
$@
END
