#!/bin/bash -e

if ! [ "$MESON_BUILD_ROOT" ]; then
    echo "This can only be run via meson, exiting!"
    exit 1
fi

PKGVER=$1-$2
DEST=${MESON_BUILD_ROOT}
DIST=$DEST/dist/$PKGVER
SRC=${MESON_SOURCE_ROOT}


cd "${MESON_SOURCE_ROOT}"
mkdir -p "${DIST}"

ginst() {
	local pattern f
	for pattern; do
		for f in ${pattern#/}; do # let shell expand the pattern
			# only create dir if needed
			if [ "${f%/*}" != "$f" ]; then
				mkdir -p "${DIST}/${f%/*}"
			fi
			cp -rv "$SRC"/$f "${DIST}/${f%/*}"
		done
	done
}

for x in $(git ls-files); do
	ginst "$x"
done

mkdir -p "${DIST}"/.cargo
cargo vendor vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > "${DIST}"/.cargo/config
cp -r vendor "${DIST}"

# packaging
cd "${DEST}"/dist
tar cJvf $PKGVER.tar.xz $PKGVER

#if type gpg; then
#	gpg --armor --detach-sign $PKGVER.tar.xz
#	gpg --verify $PKGVER.tar.xz.asc $PKGVER.tar.xz
#fi
