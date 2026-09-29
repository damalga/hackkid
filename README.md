# Hackkid

Juego de terminal en Rust. Raycaster estilo Wolfenstein 3D renderizado con `crossterm` sobre half-block chars (`▄`). Despiertas en un hospital abandonado en la ciudad de Copenlada, sin memoria clara.

Diseño completo: [`GAMEDESIGN.md`](GAMEDESIGN.md).

## Requisitos

- **Rust** ≥ 1.85 (edition 2024). Instalar con [rustup](https://rustup.rs):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Terminal** con soporte truecolor + captura de ratón (Alacritty, Kitty, WezTerm, foot, iTerm2, Windows Terminal). Tamaño mínimo recomendado: **120×40** celdas.
- **Linux**: dependencias de sistema para `rodio` (audio):
  ```sh
  # Debian / Ubuntu
  sudo apt install libasound2-dev pkg-config
  # Arch / Manjaro
  sudo pacman -S alsa-lib pkgconf
  # Fedora
  sudo dnf install alsa-lib-devel pkgconf
  ```
- **macOS / Windows**: sin dependencias extra.

## Instalación

```sh
git clone https://github.com/damalga/hackkid.git
cd hackkid
cargo build --release
./target/release/hackkid
```

Para iterar en desarrollo:
```sh
cargo run --release
```

`--release` es importante: el raycaster + blit por frame necesita optimización para ir fluido.

## Controles

| Tecla | Acción |
|-------|--------|
| W / S | Avanzar / retroceder |
| A / D | Girar |
| Rueda ratón | Correr (2× velocidad) |
| X | Interactuar (recoger, dialogar, sentarse, tumbarse, puertas) |
| I | Abrir/cerrar mochila |
| ↑ ↓ | Navegar slot (mochila abierta) |
| Enter | Usar objeto seleccionado / menú |
| T | Tirar objeto (mochila) |
| Z | Dormir (tumbado) |
| H | Ocultarse |
| M | Silenciar música |
| Esc / P | Apagar portátil |
| Ctrl+S | Guardar (`save.json`) |
| Ctrl+L | Cargar (`save.json`) |
| Ctrl+C | Salir |

## Estructura del código

```
src/
├── main.rs           bootstrap + game loop
├── audio/            rodio player + estado dinámico (drone ambiente)
├── engine/           raycaster DDA, renderer, mapa, sprites
├── game/             world, player, stats, input, action dispatch, save/load
├── objects/          camas, sofás, puertas, NPCs, fluorescentes, enchufes, etc.
├── equippables/      mochila (extensible a más equipables)
└── ui/               HUD (4 cajas), overlay mensajes, menús, laptop, techo
```

## Guardado

Partidas persisten en `save.json` en el directorio de trabajo. `Ctrl+S` guarda, `Ctrl+L` carga.

## Cross-compilación para Raspberry Pi CM4

CM4 = Cortex-A72 (ARMv8 64-bit). Requiere [`cross`](https://github.com/cross-rs/cross) + Docker.

```sh
cargo install cross --git https://github.com/cross-rs/cross
```

Crear `Cross.toml` en la raíz con las dependencias de ALSA en el contenedor:

```toml
[target.aarch64-unknown-linux-gnu]
pre-build = [
    "dpkg --add-architecture $CROSS_DEB_ARCH",
    "apt-get update && apt-get install --assume-yes libasound2-dev:$CROSS_DEB_ARCH pkg-config",
]
```

Compilar:
```sh
cross build --release --target aarch64-unknown-linux-gnu
scp target/aarch64-unknown-linux-gnu/release/hackkid pi@<cm4-ip>:~/
```

Para RPi OS 32-bit, usar `armv7-unknown-linux-gnueabihf` en su lugar.

## Licencia

Sin licencia definida por ahora. Uso personal / no distribuido.
