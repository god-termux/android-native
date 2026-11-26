#!/data/data/com.termux/files/usr/bin/bash
# Безоконный прогон сценария в harness. Использование:
#   run.sh [-c client_cmd]... 'wait 6' 'shot a' 'motion 100 100' ...   (shot NAME -> $T/NAME.png)
# Окружение: SO (по умолчанию ~/cc-android-dev/debug/libcosmic_comp.so), SIZE (по умолчанию 2392x1080), T (рабочий каталог).
R=/data/data/com.termux/files/home
SO=${SO:-$R/cc-android-dev/debug/libcosmic_comp.so}
SIZE=${SIZE:-2392x1080}
T=${T:-$TMPDIR/ct}
CLIENTS=()
while [ "$1" = "-c" ]; do CLIENTS+=("$2"); shift 2; done
rm -rf "$T"; mkdir -p "$T/xdg" "$T/home"
[ -n "$PRE" ] && eval "$PRE"
cp "$SO" "$T/comp.so"
patchelf --set-rpath "$R/.termux/cosmic-lib:/data/data/com.termux/files/usr/lib" "$T/comp.so"
SHOTS=()
{
  for l in "$@"; do
    case "$l" in
      "shot "*) n=${l#shot }; SHOTS+=("$n"); echo "shot $T/$n.ppm" ;;
      *) echo "$l" ;;
    esac
  done
  echo quit
} > "$T/script.txt"
( export XDG_RUNTIME_DIR=$T/xdg HOME=$T/home COSMIC_HEADLESS=$SIZE RUST_LOG=${RUST_LOG:-warn}
  # клиенты стартуют после появления сокета
  ( for i in $(seq 1 60); do ls "$T"/xdg/wayland-[0-9] >/dev/null 2>&1 && break; sleep 0.5; done
    S=$(ls "$T"/xdg/ | grep -E '^wayland-[0-9]+$' | head -1)
    for c in "${CLIENTS[@]}"; do ( WAYLAND_DISPLAY=$S bash -c "$c" > "$T/client-$RANDOM.log" 2>&1 & ); done ) &
  timeout 180 "$R/android-native/test/harness" "$T/comp.so" < "$T/script.txt" 2>&1 )
for n in "${SHOTS[@]}"; do [ -f "$T/$n.ppm" ] && python3 -I "$R/android-native/test/ppm2png.py" "$T/$n.ppm" "$T/$n.png" 2 && echo "PNG: $T/$n.png"; done
echo "лог композитора: $T/xdg/cosmic-comp.log"
