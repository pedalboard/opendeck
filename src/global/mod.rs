use int_enum::IntEnum;

use crate::ChannelOrAll;

pub mod parser;
pub mod renderer;

#[derive(Default)]
pub struct GlobalPreset {
    pub current: usize,
    force_value_refresh: bool,
    enable_midi_chnage: bool,
    preserve_preset: bool,
}

impl GlobalPreset {
    pub fn set(&mut self, index: &PresetIndex, value: &u16) {
        match index {
            PresetIndex::Active => self.current = *value as usize,
            PresetIndex::ForceValueRefresh => self.force_value_refresh = *value > 0,
            PresetIndex::EnableMidiChange => self.enable_midi_chnage = *value > 0,
            PresetIndex::Preservation => self.preserve_preset = *value > 0,
        }
    }
    pub fn get(&mut self, index: &PresetIndex) -> u16 {
        match index {
            PresetIndex::Active => self.current as u16,
            PresetIndex::ForceValueRefresh => self.force_value_refresh.into(),
            PresetIndex::EnableMidiChange => self.enable_midi_chnage.into(),
            PresetIndex::Preservation => self.preserve_preset.into(),
        }
    }
}

#[derive(Default)]
pub struct GlobalMidi {
    standard_note_off: bool,
    running_status: bool,
    din_to_usb_thru: bool,
    din_midi_state: bool,
    usb_to_din_thru: bool,
    usb_to_usb_thru: bool,
    usb_to_ble_thru: bool,
    din_to_din_thru: bool,
    din_to_ble_thru: bool,
    ble_midi_state: bool,
    ble_to_din_thru: bool,
    ble_to_usb_thru: bool,
    ble_to_ble_thru: bool,
    use_global_midi_channel: bool,
    global_midi_channel: ChannelOrAll,
    send_midi_clock: bool,
}

impl GlobalMidi {
    pub fn set(&mut self, index: &MidiIndex, value: &u16) {
        match index {
            MidiIndex::StandardNoteOff => self.standard_note_off = *value > 0,
            MidiIndex::RunningStatus => self.running_status = *value > 0,
            MidiIndex::DINtoUSBthru => self.din_to_usb_thru = *value > 0,
            MidiIndex::DINMIDIstate => self.din_midi_state = *value > 0,
            MidiIndex::USBtoDINthru => self.usb_to_din_thru = *value > 0,
            MidiIndex::USBtoUSBthru => self.usb_to_usb_thru = *value > 0,
            MidiIndex::USBtoBLEthru => self.usb_to_ble_thru = *value > 0,
            MidiIndex::DINtoDINthru => self.din_to_din_thru = *value > 0,
            MidiIndex::DINtoBLEthru => self.din_to_ble_thru = *value > 0,
            MidiIndex::BLEMIDIstate => self.ble_midi_state = *value > 0,
            MidiIndex::BLEtoDINthru => self.ble_to_din_thru = *value > 0,
            MidiIndex::BLEtoUSBthru => self.ble_to_usb_thru = *value > 0,
            MidiIndex::BLEtoBLEthru => self.ble_to_ble_thru = *value > 0,
            MidiIndex::UseGlobalMIDIchannel => self.use_global_midi_channel = *value > 0,
            MidiIndex::SendMIDIclock => self.send_midi_clock = *value > 0,
            MidiIndex::GlobalMIDIchannel => {
                self.global_midi_channel = ChannelOrAll::from(*value);
            }
        }
    }
    pub fn get(&mut self, index: &MidiIndex) -> u16 {
        match index {
            MidiIndex::StandardNoteOff => self.standard_note_off.into(),
            MidiIndex::RunningStatus => self.running_status.into(),
            MidiIndex::DINtoUSBthru => self.din_to_usb_thru.into(),
            MidiIndex::DINMIDIstate => self.din_midi_state.into(),
            MidiIndex::USBtoDINthru => self.usb_to_din_thru.into(),
            MidiIndex::USBtoUSBthru => self.usb_to_usb_thru.into(),
            MidiIndex::USBtoBLEthru => self.usb_to_ble_thru.into(),
            MidiIndex::DINtoDINthru => self.din_to_din_thru.into(),
            MidiIndex::DINtoBLEthru => self.din_to_ble_thru.into(),
            MidiIndex::BLEMIDIstate => self.ble_midi_state.into(),
            MidiIndex::BLEtoDINthru => self.ble_to_din_thru.into(),
            MidiIndex::BLEtoUSBthru => self.ble_to_usb_thru.into(),
            MidiIndex::BLEtoBLEthru => self.ble_to_ble_thru.into(),
            MidiIndex::UseGlobalMIDIchannel => self.use_global_midi_channel.into(),
            MidiIndex::SendMIDIclock => self.send_midi_clock.into(),
            MidiIndex::GlobalMIDIchannel => self.global_midi_channel.into(),
        }
    }
}

#[derive(IntEnum)]
#[repr(u8)]
pub enum GlobalSectionId {
    Midi = 0,
    // Reserved = 1,
    Presets = 2,
}

#[derive(Debug, Clone, PartialEq, IntEnum, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum PresetIndex {
    Active = 0,
    Preservation = 1,
    ForceValueRefresh = 2,
    EnableMidiChange = 3,
}

#[derive(Debug, Clone, PartialEq, IntEnum, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum MidiIndex {
    StandardNoteOff = 0x0,
    RunningStatus = 0x1,
    DINtoUSBthru = 0x2,
    DINMIDIstate = 0x3,
    USBtoDINthru = 0x4,
    USBtoUSBthru = 0x5,
    USBtoBLEthru = 0x6,
    DINtoDINthru = 0x7,
    DINtoBLEthru = 0x8,
    BLEMIDIstate = 0x9,
    BLEtoDINthru = 0xA,
    BLEtoUSBthru = 0xB,
    BLEtoBLEthru = 0xC,
    UseGlobalMIDIchannel = 0xD,
    GlobalMIDIchannel = 0xE,
    SendMIDIclock = 0xF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GlobalSection {
    Midi(MidiIndex, u16),
    Presets(PresetIndex, u16),
}
