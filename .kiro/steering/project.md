# OpenDeck Pedalboard Project

## Architecture

- `opendeck` crate: hardware-independent protocol library, `#![no_std]`, uses `heapless`
- `pedalboard-midi`: RP2040 firmware using RTIC, consumes `opendeck` crate
- `pedalboard-protocol`: shared config types + MIDI-CI PE framing (`#![no_std]`, `postcard` serde)
- `pedalboard-cli`: CLI tool for config upload (OpenDeck SysEx + PE per-preset)
- `pedalboard-bridge`: Go WebSocket↔MIDI bridge on CM5 (raw ALSA device I/O, auto-reconnect)
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

## Development Workflow

### Repositories

All repos live under `/home/laenzi/projects/gh/pedalboard/`:

| Repo | Language | Purpose | Runs on |
|------|----------|---------|---------|
| `pedalboard-protocol` | Rust (`#![no_std]`) | Shared config types + MIDI-CI PE framing | library |
| `pedalboard-midi` | Rust (RTIC) | RP2040 firmware | RP2040 |
| `pedalboard-cli` | Rust | Config upload tool (OpenDeck SysEx + PE per-preset) | dev machine |
| `pedalboard-bridge` | Go | WebSocket↔MIDI bridge (raw ALSA I/O, auto-reconnect) | CM5 |

### Local Setup

- Dev machine: Arch Linux, GPG SSH key (subkey 7C71F5DC)
- Test host: Raspberry Pi CM5 (`ssh laenzi@cm5-dev.home`)
- All repos cloned under `/home/laenzi/projects/gh/pedalboard/`
- Pre-commit hooks installed in all repos

### Development Loop

1. **Edit locally** — pre-commit hooks validate (fmt, clippy/vet, test/build)
2. **Push to main directly** — no PRs for now, solo project
3. **CI validates** on GitHub Actions (all repos have `ci.yml`)
4. **Deploy**:
   - Firmware: `cd pedalboard-midi && make flash` (via bridge DFU) or `make flash-probe` (via SWD for dev)
   - Bridge: `cd pedalboard-bridge && make deploy` (push, pull on CM5, build, restart service)
   - CLI: just `cargo build` (runs locally)

### Version Convention

All binaries embed `<semver>-<git-hash>`. Uncommitted builds show `<semver>-<hash>+dev`.

### Testing Without Commit

`make flash-probe` builds and flashes without needing to commit. Version shows `+dev` suffix.

### Bridge Endpoints

- `/config` — OpenDeck web UI WebSocket
- `/raw` — raw SysEx passthrough for PE
- `/dfu` — firmware flash

### Key Rules

- All code changes happen on the dev machine, never edit directly on CM5
- Bridge auto-reconnects on firmware reboot/flash
- Never use `--no-verify` on commits (CI catches what hooks miss)
- `pedalboard-midi` uses `--config` patches for local opendeck/protocol deps — **never modify `Cargo.toml` `[patch]` sections**
- Go binary on CM5 not in PATH — use `/usr/local/go/bin/go`

## Key Learnings

### Architecture Decision: Protocol approach
- Currently hybrid: OpenDeck `0x43` for MIDI config + UI, custom `0x44` for labels/extensions
- CLI (`pedalboard-cli`) uses both protocols via bridge WebSocket for gitops workflow
- If custom extensions keep growing, consider switching to direct config upload (YAML → binary → flash) instead of per-field SysEx mutation
- Keep OpenDeck for UI/debugging even if primary config path changes

### Architecture Decision: Direct action model (next evolution)
- Primary path: YAML defines actions directly (on_press/on_release → MIDI messages + LED state)
- Firmware executes actions and updates LEDs locally — no MIDI feedback loop
- Supports multi-message sequences, radio groups, toggle, momentary, level
- Config uploaded as binary blob to flash, firmware reads on boot
- OpenDeck remains as secondary mode for web UI / live debugging / compatibility
- This replaces the current approach of mapping YAML fields to OpenDeck SysEx parameters

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
- Factory reset erases flash but does NOT reboot (differs from upstream which reboots after 1s). In-memory config stays until power cycle. CLI can upload new config immediately after reset.

### RTIC
- `or_else` on `Option` short-circuits — don't use for polling multiple inputs
- Use `heapless::Vec` to collect all events per cycle
- `Mono::delay().await` in loop is less precise than `spawn_after` — both encoders still work at 1ms
- USB send task must loop waiting for configured state, not return early
