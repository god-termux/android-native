#!/usr/bin/env bash
# Готовит ./deps: копирует исходники зависимостей из кэша cargo и накладывает патчи для Android.
# Запускайте после первой неудачной `cargo fetch` в cosmic-comp (чтобы git-зависимости были в ~/.cargo).
set -euo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd)
CG=${CARGO_HOME:-$HOME/.cargo}
mkdir -p "$HERE/deps"
# патчи сделаны с префиксами a/ b/ от корня крейта
one() { name=$1; src=$2
  rm -rf "$HERE/deps/$name"; cp -r "$src" "$HERE/deps/$name"; rm -rf "$HERE/deps/$name/.git"
  ( cd "$HERE/deps/$name" && patch -p1 --no-backup-if-mismatch < "$HERE/patches/$name.patch" ) >/dev/null
  echo "ok $name"; }
one window_clipboard "$(ls -d $CG/git/checkouts/window_clipboard-*/f68595e)"
one softbuffer       "$(ls -d $CG/git/checkouts/softbuffer-*/c2b2c19)"
one smithay          "$(ls -d $CG/git/checkouts/smithay-*/8a27023)"
one wayland-cursor   "$(ls -d $CG/registry/src/*/wayland-cursor-0.31.14)"
