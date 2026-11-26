# android-native

> **Статус: indev (в активной разработке).** Проект экспериментальный и нестабильный: интерфейсы, патчи и порядок сборки
> могут меняться без предупреждения, готового к использованию результата пока нет.

Сборка COSMIC-композитора (`cosmic-comp`) под Android (bionic) прямо на телефоне в Termux, чтобы он рисовал через
Android EGL на GPU, а не программно (llvmpipe) во вложенном режиме.

Состояние (indev): композитор собирается под `aarch64-linux-android`, рисует рабочий стол COSMIC на GPU (EGL на Mali)
и принимает мышь, клавиатуру и касания. Проверено безоконно: harness в обычном процессе Termux (рендер в pbuffer и в
`ANativeWindow` из `AImageReader`), снимки кадров, `cosmic-term`, панель и док. Не проверено: экран мода в самом
приложении (SurfaceView, жизненный цикл). Ветка форка: `android-native` в
[god-termux/cosmic-comp](https://github.com/god-termux/cosmic-comp); мод и экран: `god/mods/cosmic` в
[god-termux/god-termux](https://github.com/god-termux/god-termux) (ветка `cosmic-native`).

## Для какого устройства написано

Весь функционал написан и проверяется под одно устройство:

- ядро Linux `6.12.69-android16-6-g014ca48a80ee-ab15889839-4k` (aarch64);
- Android 16;
- прошивка `BP2A.250605.031.A3` (модель `2510DRA23E`, SoC MediaTek MT6789, GPU Mali-G57).

В будущем код будет адаптироваться под другие ядра, версии Android и прошивки. Пока на других устройствах
работоспособность не гарантируется.

## Что здесь

- `scripts/build-libs.sh` — собирает под bionic `libudev-zero`, `libinput` 1.28.1, `libdisplay-info` 0.3.0.
- `patches/` — патчи зависимостей, не знающих про Android: `window_clipboard`, `softbuffer`, `wayland-cursor`, `smithay`.
- `scripts/setup-deps.sh` — копирует эти крейты из кэша cargo в `deps/` и накладывает патчи.
- `test/` — безоконные тесты: `harness.c` (грузит `libcosmic_comp.so`, команды со stdin), `run.sh` (сценарий + PNG),
  `desktop.sh` (весь стол COSMIC), `ppm2png.py`. Режим окна: `HARNESS_WINDOW=1`.
- `cargo-config.example.toml` — пример `.cargo/config.toml` для `cosmic-comp` (пути `[patch]` на `deps/`).

## Как собрать

```sh
git clone https://github.com/god-termux/android-native
git clone -b android-native https://github.com/god-termux/cosmic-comp
./android-native/scripts/build-libs.sh                  # C-библиотеки
cd cosmic-comp && cargo fetch || true                   # положить git-зависимости в ~/.cargo
../android-native/scripts/setup-deps.sh                 # deps/ с патчами
cp ../android-native/cargo-config.example.toml .cargo/config.toml
export PKG_CONFIG_PATH=$HOME/android-libs/prefix/lib/pkgconfig
cargo check --no-default-features
```

Rust в Termux уже собирает под `aarch64-linux-android`, кросс-компиляция не нужна.

## Тесты без окна

```sh
cd android-native/test && clang -O1 harness.c -o harness -ldl -lpthread
./run.sh -c "env FHS_DEBIAN=1 ... /path/to/cosmic-term" 'wait 12' 'shot a' 'motion 1300 500' 'button 0 1' 'button 0 0'
./desktop.sh 25 desk                   # панель, фон, уведомления, osd; снимок desk.png
HARNESS_WINDOW=1 ./run.sh 'wait 8' 'shot w' 'lost' 'newwin' 'shot w2'   # путь с ANativeWindow
```
Переменные: `SO` (путь к `libcosmic_comp.so`), `SIZE` (по умолчанию 2392x1080), `COSMIC_SCALE` (масштаб интерфейса).
