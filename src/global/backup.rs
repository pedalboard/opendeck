use crate::global::{GlobalMidi, GlobalPreset, GlobalSection, MidiIndex, PresetIndex};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct GlobalPresetBackupIterator {
    preset_index: PresetIndex,
    done: bool,
}

impl GlobalPresetBackupIterator {
    pub fn new() -> Self {
        Self {
            preset_index: PresetIndex::Active,
            done: false,
        }
    }
    pub fn next(&mut self, global_preset: &GlobalPreset) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let key = self.preset_index;
        let value = match self.preset_index {
            PresetIndex::Active => {
                self.preset_index = PresetIndex::Preservation;
                global_preset.current as u16
            }
            PresetIndex::Preservation => {
                self.preset_index = PresetIndex::ForceValueRefresh;
                global_preset.preserve_preset as u16
            }
            PresetIndex::ForceValueRefresh => {
                self.preset_index = PresetIndex::EnableMidiChange;
                global_preset.force_value_refresh as u16
            }
            PresetIndex::EnableMidiChange => {
                self.done = true;
                global_preset.enable_midi_chnage as u16
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Global(GlobalSection::Presets(key, value)),
            new_values,
        ))
    }
}

impl Default for GlobalPresetBackupIterator {
    fn default() -> Self {
        Self::new()
    }
}
pub struct GlobalMidiBackupIterator {
    midi_index: MidiIndex,
    done: bool,
}

impl GlobalMidiBackupIterator {
    pub fn new() -> Self {
        Self {
            midi_index: MidiIndex::StandardNoteOff,
            done: false,
        }
    }
    pub fn next(&mut self, global_midi: &GlobalMidi) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let key = self.midi_index;
        let value = match self.midi_index {
            MidiIndex::StandardNoteOff => {
                self.midi_index = MidiIndex::RunningStatus;
                global_midi.standard_note_off as u16
            }
            MidiIndex::RunningStatus => {
                self.midi_index = MidiIndex::DINtoUSBthru;
                global_midi.running_status as u16
            }
            MidiIndex::DINtoUSBthru => {
                self.midi_index = MidiIndex::DINMIDIstate;
                global_midi.din_to_usb_thru as u16
            }
            MidiIndex::DINMIDIstate => {
                self.midi_index = MidiIndex::USBtoDINthru;
                global_midi.din_midi_state as u16
            }
            MidiIndex::USBtoDINthru => {
                self.midi_index = MidiIndex::USBtoUSBthru;
                global_midi.usb_to_din_thru as u16
            }
            MidiIndex::USBtoUSBthru => {
                self.midi_index = MidiIndex::USBtoBLEthru;
                global_midi.usb_to_usb_thru as u16
            }
            MidiIndex::USBtoBLEthru => {
                self.midi_index = MidiIndex::DINtoDINthru;
                global_midi.usb_to_ble_thru as u16
            }
            MidiIndex::DINtoDINthru => {
                self.midi_index = MidiIndex::DINtoBLEthru;
                global_midi.din_to_din_thru as u16
            }
            MidiIndex::DINtoBLEthru => {
                self.midi_index = MidiIndex::BLEMIDIstate;
                global_midi.din_to_ble_thru as u16
            }
            MidiIndex::BLEMIDIstate => {
                self.midi_index = MidiIndex::BLEtoDINthru;
                global_midi.ble_midi_state as u16
            }
            MidiIndex::BLEtoDINthru => {
                self.midi_index = MidiIndex::BLEtoUSBthru;
                global_midi.ble_to_din_thru as u16
            }
            MidiIndex::BLEtoUSBthru => {
                self.midi_index = MidiIndex::BLEtoBLEthru;
                global_midi.ble_to_usb_thru as u16
            }
            MidiIndex::BLEtoBLEthru => {
                self.midi_index = MidiIndex::UseGlobalMIDIchannel;
                global_midi.ble_to_ble_thru as u16
            }
            MidiIndex::UseGlobalMIDIchannel => {
                self.midi_index = MidiIndex::GlobalMIDIchannel;
                global_midi.use_global_midi_channel as u16
            }
            MidiIndex::GlobalMIDIchannel => {
                self.midi_index = MidiIndex::SendMIDIclock;
                global_midi.global_midi_channel.into()
            }
            MidiIndex::SendMIDIclock => {
                self.done = true;
                global_midi.send_midi_clock.into()
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Global(GlobalSection::Midi(key, value)),
            new_values,
        ))
    }
}

impl Default for GlobalMidiBackupIterator {
    fn default() -> Self {
        Self::new()
    }
}
