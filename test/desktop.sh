#!/data/data/com.termux/files/usr/bin/bash
# Безоконный тест рабочего стола COSMIC (панель, фон, уведомления, osd) на нашем композиторе.
#   desktop.sh [секунд_до_снимка=25] [имя_снимка=desktop] [доп. команды harness...]
R=/data/data/com.termux/files/home
D=/data/data/com.termux/files/usr/debian
WAIT=${1:-25}; NAME=${2:-desktop}; shift 2 2>/dev/null
export T=${T:-$TMPDIR/ct}
mkdir -p "$T"; BUSDIR=$T.bus; rm -rf "$BUSDIR"; mkdir -p "$BUSDIR"
OUT=$(env FHS_DEBIAN=1 XDG_RUNTIME_DIR=$BUSDIR "$D/usr/bin/dbus-daemon" --session --fork --print-address --print-pid 2>/dev/null)
ADDR=$(printf '%s\n' "$OUT" | sed -n 1p); BPID=$(printf '%s\n' "$OUT" | sed -n 2p)
ENVP="env FHS_DEBIAN=1 NO_AT_BRIDGE=1 DBUS_SESSION_BUS_ADDRESS=$ADDR XDG_DATA_DIRS=$D/usr/share XKB_CONFIG_ROOT=$D/usr/share/X11/xkb GDK_BACKEND=wayland"
ARGS=()
for c in cosmic-settings-daemon cosmic-bg cosmic-notifications cosmic-panel cosmic-osd; do
  [ -x "$D/usr/bin/$c" ] && ARGS+=(-c "sleep $([ $c = cosmic-panel ] && echo 4 || echo 1); $ENVP $D/usr/bin/$c")
done
"$R/android-native/test/run.sh" "${ARGS[@]}" "wait $WAIT" "shot $NAME" "$@"
[ -n "$BPID" ] && kill "$BPID" 2>/dev/null
