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
    use_global_channel: bool,
    global_channel: ChannelOrAll,
    standard_note_off: bool,
    din_midi: bool,
    ble_midi: bool,
    usb_to_usb_through: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GlobalSection {
    Midi(u16, u16),
    Presets(PresetIndex, u16),
}
