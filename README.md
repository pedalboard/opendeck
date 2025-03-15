# OpenDeck protocol

Rust crate of an implementation of the [OpenDeck MIDI Sysex Protocol](https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration)

## TODO

The implementation is not yet complete. The following features are missing:

## Missing handler functions

* LED support
* global settings
* handlers with effect on global settings
  * BPM
  * Preset Change

## configuration improvements

* value_size feature for configuration protocol
* iterator for configuration output messages
* use midi2 sysex messages in API
