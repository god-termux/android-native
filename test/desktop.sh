#!/data/data/com.termux/files/usr/bin/bash
# Безоконный тест рабочего стола COSMIC (панель, фон, уведомления, osd) на нашем композиторе.
#   desktop.sh [секунд_до_снимка=25] [имя_снимка=desktop] [доп. команды harness...]
R=/data/data/com.termux/files/home
D=/data/data/com.termux/files/usr/debian
WAIT=${1:-25}; NAME=${2:-desktop}; shift 2 2>/dev/null
export T=${T:-$TMPDIR/ct}
mkdir -p "$T"
# шина на фиксированном адресе $XDG_RUNTIME_DIR/bus — композитору (без переменных окружения) она находится по умолчанию
export PRE='mkdir -p "$T/xdg"; env FHS_DEBIAN=1 XDG_RUNTIME_DIR="$T/xdg" "'$D'/usr/bin/dbus-daemon" --session --address="unix:path=$T/xdg/bus" --fork --print-pid > "$T/dbus.pid" 2>/dev/null'
ADDR="unix:path=$T/xdg/bus"
# конфиг панели: добавить апплет экранной клавиатуры (как это делает лаунчер)
CFG=$T/home/.config/cosmic/com.system76.CosmicPanel.Panel/v1
# автопоказ клавиатуры при фокусе в текстовом поле (настройка-состояние cosmic-osk)
OSKS=$T/home/.local/state/cosmic/com.system76.CosmicOSK/v1
export PRE="$PRE; mkdir -p \"$OSKS\"; printf true > \"$OSKS/ime_activation\"; mkdir -p \"$CFG\"; sed 's/\"com.system76.CosmicAppletInputSources\"/\"com.system76.CosmicAppletOsk\",\"com.system76.CosmicAppletInputSources\"/' $D/usr/share/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings > \"$CFG/plugins_wings\""
ENVP="env FHS_DEBIAN=1 NO_AT_BRIDGE=1 DBUS_SESSION_BUS_ADDRESS=$ADDR XDG_CONFIG_HOME=$T/home/.config XDG_STATE_HOME=$T/home/.local/state XDG_DATA_DIRS=$D/usr/share XKB_CONFIG_ROOT=$D/usr/share/X11/xkb GDK_BACKEND=wayland RUST_LOG=${CRL:-warn}"
ARGS=()
for c in cosmic-settings-daemon cosmic-bg cosmic-notifications cosmic-panel cosmic-osd cosmic-osk; do
  [ -x "$D/usr/bin/$c" ] && ARGS+=(-c "sleep $([ $c = cosmic-panel ] && echo 4 || echo 1); $ENVP $D/usr/bin/$c")
done
# OSK_CMD: подкоманда cosmic-osk (show/hide), запускаемая через OSK_DELAY секунд после старта
[ -n "${OSK_CMD:-}" ] && ARGS+=(-c "sleep ${OSK_DELAY:-30}; $ENVP $D/usr/bin/cosmic-osk $OSK_CMD")
"$R/android-native/test/run.sh" "${ARGS[@]}" "wait $WAIT" "shot $NAME" "$@"
[ -f "$T/dbus.pid" ] && kill "$(cat "$T/dbus.pid")" 2>/dev/null
