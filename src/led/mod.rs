use crate::ChannelOrAll;
use int_enum::IntEnum;
use midi_types::Value7;

pub mod parser;
pub mod renderer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Led {
    color_testing: Color,
    blink_testing: bool,
    activation_id: Value7,
    activation_value: Value7,
    rgb_enabled: bool,
    control_type: ControlType,
    channel: ChannelOrAll,
}

impl Led {
    pub fn new(midi_id: Value7) -> Self {
        Led {
            color_testing: Color::default(),
            blink_testing: false,
            activation_id: midi_id,
            activation_value: Value7::new(0),
            rgb_enabled: true,
            control_type: ControlType::default(),
            channel: ChannelOrAll::default(),
        }
    }
    pub fn set(&mut self, section: &LedSection) {
        match section {
            LedSection::ColorTesting(v) => self.color_testing = *v,
            LedSection::BlinkTesting(v) => self.blink_testing = *v,
            LedSection::RGBEnabled(v) => self.rgb_enabled = *v,
            LedSection::ControlType(v) => self.control_type = *v,
            LedSection::ActivationId(v) => self.activation_id = *v,
            LedSection::ActivationValue(v) => self.activation_value = *v,
            LedSection::Channel(v) => self.channel = *v,
            LedSection::Global(_) => {}
        }
    }
    pub fn get(&self, section: &LedSection) -> u16 {
        match section {
            LedSection::ColorTesting(_) => self.color_testing.into(),
            LedSection::BlinkTesting(_) => self.blink_testing.into(),
            LedSection::RGBEnabled(_) => self.rgb_enabled.into(),
            LedSection::ControlType(_) => self.control_type.into(),
            LedSection::ActivationId(_) => {
                let v: u8 = self.activation_id.into();
                v as u16
            }
            LedSection::ActivationValue(_) => {
                let v: u8 = self.activation_value.into();
                v as u16
            }
            LedSection::Channel(_) => self.channel.into(),
            LedSection::Global(_) => 0,
        }
    }
}

impl Default for Led {
    fn default() -> Self {
        Led::new(Value7::new(0x00))
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LedSection {
    ColorTesting(Color),
    BlinkTesting(bool),
    Global(u16),
    ActivationId(Value7),
    ActivationValue(Value7),
    RGBEnabled(bool),
    ControlType(ControlType),
    Channel(ChannelOrAll),
}
