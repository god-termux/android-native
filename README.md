# android-native

> **Статус: indev (в активной разработке).** Проект экспериментальный и нестабильный: интерфейсы, патчи и порядок сборки
> могут меняться без предупреждения, готового к использованию результата пока нет.

Сборка COSMIC-композитора (`cosmic-comp`) под Android (bionic) прямо на телефоне в Termux, чтобы он рисовал через
Android EGL на GPU, а не программно (llvmpipe) во вложенном режиме.

Состояние: `cargo check` под `aarch64-linux-android` проходит. Бэкенд вывода (EGL в `ANativeWindow`) и ввода есть;
точка входа `.so` для приложения и подключение к god-termux — в работе. Подробности — в ветке `android-native`
репозитория [god-termux/cosmic-comp](https://github.com/god-termux/cosmic-comp).

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
