use crate::ChannelOrAll;
use int_enum::IntEnum;

pub mod parser;
pub mod renderer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Led {
    color_testing: Color,
    blink_testing: bool,
    activation_id: u8,
    activation_value: u8,
    rgb_enabled: bool,
    control_type: ControlType,
    channel: ChannelOrAll,
}

#[derive(Default)]
pub struct GlobalLed {
    blink_with_midi_clock: bool,
    startup_animtation: bool,
    midi_program_change_offset: bool,
}

#[derive(Debug, Clone, PartialEq, IntEnum, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum LedIndex {
    BlinkWithMIDIClock = 0,
    // FadeSpeed = 1, // not in use anymore
    EnableStartupAnimation = 2,
    UseMidiProgramChangeOffset = 3,
}

// FIXME call global led settings
impl GlobalLed {
    pub fn set(&mut self, index: LedIndex, value: &u16) {
        match index {
            LedIndex::BlinkWithMIDIClock => self.blink_with_midi_clock = *value > 0,
            LedIndex::EnableStartupAnimation => self.startup_animtation = *value > 0,
            LedIndex::UseMidiProgramChangeOffset => self.midi_program_change_offset = *value > 0,
        }
    }
    pub fn get(&mut self, index: &LedIndex) -> u16 {
        match index {
            LedIndex::BlinkWithMIDIClock => self.blink_with_midi_clock.into(),
            LedIndex::EnableStartupAnimation => self.startup_animtation.into(),
            LedIndex::UseMidiProgramChangeOffset => self.midi_program_change_offset.into(),
        }
    }
}

impl Led {
    pub fn new(midi_id: u8) -> Self {
        Led {
            color_testing: Color::default(),
            blink_testing: false,
            activation_id: midi_id,
            activation_value: 0,
            rgb_enabled: true,
            control_type: ControlType::default(),
            channel: ChannelOrAll::default(),
        }
    }
    pub fn set(&mut self, section: LedSection) {
        match section {
            LedSection::ColorTesting(v) => self.color_testing = v,
            LedSection::BlinkTesting(v) => self.blink_testing = v,
            LedSection::RGBEnabled(v) => self.rgb_enabled = v,
            LedSection::ControlType(v) => self.control_type = v,
            LedSection::ActivationId(v) => self.activation_id = v,
            LedSection::ActivationValue(v) => self.activation_value = v,
            LedSection::Channel(v) => self.channel = v,
            LedSection::Global(_) => {}
        }
    }
    pub fn get(&self, section: LedSection) -> u16 {
        match section {
            LedSection::ColorTesting(_) => self.color_testing.into(),
            LedSection::BlinkTesting(_) => self.blink_testing.into(),
            LedSection::RGBEnabled(_) => self.rgb_enabled.into(),
            LedSection::ControlType(_) => self.control_type.into(),
            LedSection::ActivationId(_) => self.activation_id.into(),
            LedSection::ActivationValue(_) => self.activation_value.into(),
            LedSection::Channel(_) => self.channel.into(),
            LedSection::Global(_) => 0,
        }
    }
}

impl Default for Led {
    fn default() -> Self {
        Led::new(0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum Color {
    #[default]
    Off = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum ControlType {
    #[default]
    MidiInNoteSingleValue = 0,
    LocalNoteSingleValue = 1,
    MidiInCcSingleValue = 2,
    LocalCcSingleValue = 3,
    ProgramChange = 4,
    PresetChange = 5,
    MidiInNoteMultiValue = 6,
    LocalNoteMultiValue = 7,
    MidiInCcMultiValue = 8,
    LocalCcMultiValue = 9,
    Static = 10,
}

#[derive(IntEnum)]
#[repr(u8)]
enum LedSectionId {
    ColorTesting = 0,
    BlinkTesting = 1,
    Global = 2,
    ActivationId = 3,
    RGBEnabled = 4,
    ControlType = 5,
    ActivationValue = 6,
    Channel = 7,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LedSection {
    ColorTesting(Color),
    BlinkTesting(bool),
    Global(u16),
    ActivationId(u8),
    ActivationValue(u8),
    RGBEnabled(bool),
    ControlType(ControlType),
    Channel(ChannelOrAll),
}
