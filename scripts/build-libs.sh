#!/usr/bin/env bash
# Сборка C-библиотек, нужных cosmic-comp под Android (bionic), прямо в Termux.
# Результат: $PREFIX_DIR (по умолчанию ~/android-libs/prefix) — добавьте $PREFIX_DIR/lib/pkgconfig в PKG_CONFIG_PATH.
set -euo pipefail
ROOT=${ROOT:-$HOME/android-libs}
PREFIX_DIR=${PREFIX_DIR:-$ROOT/prefix}
mkdir -p "$ROOT" && cd "$ROOT"
pkg install -y meson ninja clang libevdev mtdev libseat >/dev/null

# libudev-zero: тонкая замена libudev (в Android udev нет)
[ -d libudev-zero ] || git clone -q --depth 1 https://github.com/illiliti/libudev-zero.git
make -C libudev-zero CC=clang PREFIX="$PREFIX_DIR" install

export PKG_CONFIG_PATH=$PREFIX_DIR/lib/pkgconfig

# libinput 1.28.1: bionic без index()/rindex()
[ -d libinput ] || git clone -q --depth 1 --branch 1.28.1 https://gitlab.freedesktop.org/libinput/libinput.git
( cd libinput
  sed -i 's/\brindex(/strrchr(/' src/util-strings.c
  sed -i 's/\bindex(current/strchr(current/' src/util-prop-parsers.c
  rm -rf build
  meson setup build --prefix="$PREFIX_DIR" -Dlibwacom=false -Ddebug-gui=false -Dtests=false \
    -Ddocumentation=false -Dinstall-tests=false \
    -Dc_args="-I$PREFIX_DIR/include -include strings.h" -Dc_link_args="-L$PREFIX_DIR/lib"
  ninja -C build install )

# libdisplay-info 0.3.0 (крейт libdisplay-info-sys требует <0.4; в Termux 0.4.0). Нужен hwdata/pnp.ids.
mkdir -p hwdata-stub
[ -s hwdata-stub/pnp.ids ] || curl -sL https://raw.githubusercontent.com/vcrhonek/hwdata/master/pnp.ids -o hwdata-stub/pnp.ids
[ -d libdisplay-info ] || git clone -q --depth 1 --branch 0.3.0 https://gitlab.freedesktop.org/emersion/libdisplay-info.git
( cd libdisplay-info
  sed -i "s#files('/usr/share/hwdata/pnp.ids')#files('$ROOT/hwdata-stub/pnp.ids')#" meson.build
  rm -rf build
  meson setup build --prefix="$PREFIX_DIR"
  ninja -C build install )
echo "готово: $PREFIX_DIR"
