# OpenDeck Pedalboard Project

## Architecture

- `opendeck` crate: hardware-independent protocol library, `#![no_std]`, uses `heapless`
- `pedalboard-midi`: RP2040 firmware using RTIC, consumes `opendeck` crate
- `pedalboard-hw`: KiCad hardware design (schematics, PCB)
- `pedalboard-graphics`: display UI prototype (desktop simulator)

## Hardware

- MCU: RP2040 (thumbv6m-none-eabi)
- Encoders: 2x Alps EC11E on GPIO16/17/18 (Vol) and GPIO19/20/21 (Gain)
- Buttons: 6x on GPIO2-7
- Expression pedals: 2x ADC on GPIO27/28
- LEDs: WS2812 via SPI1 (8 rings of 12 LEDs + 2 single LEDs)
- Displays: 2x SSD1327 128x128 OLED via I2C0 (addr 0x3C, 0x3D)
- MIDI: UART0 on GPIO0/1 (DIN), USB MIDI
- Debug probe: not connected to cm5-dev, flash via UF2 only

## Development Setup

- Dev machine: Arch Linux, GPG SSH key (subkey 7C71F5DC)
- Test host: Raspberry Pi CM5 (`ssh laenzi@cm5-dev.home`)
- Flash process: build on cm5 → SysEx bootloader cmd → copy UF2
- SysEx bootloader sequence: handshake first, wait for ACK, then send 0x55
- UF2 auto-mounts at `/media/laenzi/RPI-RP2` on cm5-dev

## Key Learnings

### Encoders
- `rotary-encoder-embedded` v0.5.0 breaks detection, pin to v0.3.1 (rev d1b8795)
- Default `pulses_per_step=4` requires 4 detent clicks per MIDI message — set to 1 for these encoders
- Both encoders work, but MUST have unique MIDI IDs (they default to matching index)

### OpenDeck Protocol
- `ChannelOrAll::Channel(n)` uses 0-based internal storage but 1-based wire format
- `Led::get()` returns protocol-encoded u16 values — don't roundtrip through `From<u16>` for internal use
- Use `channel_direct()` for internal access, `get()/set()` only for protocol serialization
- `Preset<B, A, E, L>` has a generic param ordering bug in one impl block (swaps A and E) — pre-existing, masked when all sizes are equal

### LED Output Handler (WIP)
- The handler `process_midi()` works in isolation (9 tests pass)
- Integration with `Config::update_outputs()` fails — `get_control_type()` doesn't return the value set via `set()`
- Root cause TBD: likely the `get()` roundtrip or field access issue
- Stashed in `git stash` on the opendeck repo

### Firmware
- Buttons work out of the box (Note On/Off)
- Encoders need: enabled=true, CC mode, pulses_per_step=1
- Analog needs: enabled=true, unique MIDI ID (offset from encoders)
- Configure all at boot in `opendeck_handler.rs` via `process_req()`
- `defmt` feature removed from opendeck dep to allow host-side testing
- Host tests use `cargo test --lib --target x86_64-unknown-linux-gnu`

### RTIC
- `or_else` on `Option` short-circuits — don't use for polling multiple inputs
- Use `heapless::Vec` to collect all events per cycle
- `Mono::delay().await` in loop is less precise than `spawn_after` — both encoders still work at 1ms
- USB send task must loop waiting for configured state, not return early

## Deployment Checklist

1. `cargo build --release` on cm5-dev
2. `elf2uf2-rs ./target/thumbv6m-none-eabi/release/pedalboard-midi`
3. `amidi -S 'F0 00 53 43 00 00 01 F7' -p hw:2,0,0 -d -t 2` (handshake + wait ACK)
4. `amidi -S 'F0 00 53 43 00 00 55 F7' -p hw:2,0,0` (bootloader mode)
5. Wait for `/media/laenzi/RPI-RP2` to appear
6. `cp .../pedalboard-midi.uf2 /media/laenzi/RPI-RP2/ && sync`

## Firmware Build (pedalboard-midi)

- The `pedalboard-midi` Makefile uses `--config` to patch the opendeck dependency at build time
- `OPENDECK_PATCH := --config 'patch."https://github.com/pedalboard/opendeck".opendeck.path="../opendeck"'`
- **Never modify `Cargo.toml` to add `[patch]` sections** — use `make build` or pass `$(OPENDECK_PATCH)` manually
- `make build` — release build with local opendeck
- `make lint` — clippy with local opendeck
- `make uf2` — build + convert to UF2
- `make bootsel` — enter bootloader via SysEx
- `make install` — copy UF2 to mounted RP2040
