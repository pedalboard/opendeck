# OpenDeck protocol

A `#![no_std]` Rust crate implementing the [OpenDeck MIDI SysEx Protocol](https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration).

## Acknowledgments

This project is built on top of the excellent [OpenDeck](https://github.com/shanteacontrols/OpenDeck) platform created by [Igor Petrović / Shantea Controls](https://shanteacontrols.com). OpenDeck is a remarkable open-source project that provides a complete, well-documented MIDI controller platform — from firmware to configurator. The quality of the protocol documentation and the thoughtful design of the SysEx configuration interface made this Rust implementation possible. We are grateful for Igor's work in making professional MIDI controller development accessible to the open hardware community.

## TODO

The implementation is not yet complete. The following features are missing:

## Missing handler functions

* LED support
* global settings currently have no influence
* handlers with effect on global settings
  * BPM
  * Preset Change
* configurable ADC max
* encoder Acceleration

## configuration improvements

* use midi2::sysex messages for input messages
* support factory reset
* support component info
* value_size feature for configuration protocol
* support controller amount > 32
