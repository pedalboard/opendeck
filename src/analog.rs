use crate::{parser::OpenDeckParseError, ChannelOrAll, MessageStatus, Section};
use int_enum::IntEnum;
use midi_types::Value14;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Analog {
    enabled: bool,
    invert_state: bool,
    message_type: AnalogMessageType,
    midi_id: Value14,
    upper_limit: Value14,
    lower_limit: Value14,
    channel: ChannelOrAll,
    lower_adc_offset: u8,
    upper_adc_offset: u8,
}

impl Analog {
    pub fn new(midi_id: Value14) -> Self {
        Analog {
            enabled: true,
            invert_state: false,
            message_type: AnalogMessageType::default(),
            channel: ChannelOrAll::default(),
            midi_id,
            lower_limit: Value14::from(u16::MIN),
            upper_limit: Value14::from(u16::MIN),
            lower_adc_offset: 0,
            upper_adc_offset: 0,
        }
    }
    pub fn set(&mut self, section: &AnalogSection) {
        match section {
            AnalogSection::MessageType(v) => self.message_type = *v,
            AnalogSection::Channel(v) => self.channel = v.clone(),
            AnalogSection::Enabled(v) => self.enabled = *v,
            AnalogSection::MidiId(v) => self.midi_id = *v,
            AnalogSection::InvertState(v) => self.invert_state = *v,
            AnalogSection::LowerCCLimit(v) => self.lower_limit = *v,
            AnalogSection::UpperCCLimit(v) => self.upper_limit = *v,
            AnalogSection::LowerADCOffset(v) => self.lower_adc_offset = *v,
            AnalogSection::UpperADCOffset(v) => self.upper_adc_offset = *v,
        }
    }
    pub fn get(&self, section: &AnalogSection) -> u16 {
        match section {
            AnalogSection::MessageType(_) => self.message_type as u16,
            AnalogSection::Channel(_) => self.channel.clone().into(),
            AnalogSection::Enabled(_) => self.enabled as u16,
            AnalogSection::MidiId(_) => self.midi_id.into(),
            AnalogSection::InvertState(_) => self.invert_state as u16,
            AnalogSection::LowerCCLimit(_) => self.lower_limit.into(),
            AnalogSection::UpperCCLimit(_) => self.upper_limit.into(),
            AnalogSection::LowerADCOffset(_) => self.lower_adc_offset.into(),
            AnalogSection::UpperADCOffset(_) => self.upper_adc_offset.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum AnalogMessageType {
    #[default]
    PotentiometerWithCCMessage7Bit = 0,
    PotentiometerWithNoteMessage = 1,
    FSR = 2,
    Button = 3,
    NRPN7 = 4,
    NRPN8 = 5,
    PitchBend = 6,
    PotentiometerWithCCMessage14Bit = 7,
}

#[allow(dead_code)]
enum AnalogSectionId {
    Enabled,
    InvertState,
    MessageType,
    MidiIdLSB,
    MidiIdMSB, // only used in 1 byte protocol
    LowerCCLimitLSB,
    LowerCCLimitMSB, // only used in 1 byte protocol
    UpperCCLimitLSB,
    UpperCCLimitMSB, // only used in 1 byte protocol
    Channel,
    LowerADCOffset,
    UpperADCOffset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnalogSection {
    Enabled(bool),
    InvertState(bool),
    MessageType(AnalogMessageType),
    MidiId(Value14),
    LowerCCLimit(Value14),
    UpperCCLimit(Value14),
    Channel(ChannelOrAll),
    LowerADCOffset(u8),
    UpperADCOffset(u8),
}

impl TryFrom<Section> for AnalogSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == AnalogSectionId::Enabled as u8 => Ok(AnalogSection::Enabled(x.value > 0)),
            x if x.id == AnalogSectionId::InvertState as u8 => {
                Ok(AnalogSection::InvertState(x.value > 0))
            }
            x if x.id == AnalogSectionId::MessageType as u8 => {
                if let Ok(mt) = AnalogMessageType::try_from(x.value) {
                    Ok(AnalogSection::MessageType(mt))
                } else {
                    Err(OpenDeckParseError::StatusError(
                        MessageStatus::NewValueError,
                    ))
                }
            }
            x if x.id == AnalogSectionId::MidiIdLSB as u8 => {
                Ok(AnalogSection::MidiId(Value14::from(x.value)))
            }
            x if x.id == AnalogSectionId::LowerCCLimitLSB as u8 => {
                Ok(AnalogSection::LowerCCLimit(Value14::from(x.value)))
            }
            x if x.id == AnalogSectionId::UpperCCLimitLSB as u8 => {
                Ok(AnalogSection::UpperCCLimit(Value14::from(x.value)))
            }
            x if x.id == AnalogSectionId::Channel as u8 => {
                Ok(AnalogSection::Channel(ChannelOrAll::from(x.value)))
            }
            x if x.id == AnalogSectionId::LowerADCOffset as u8 => {
                Ok(AnalogSection::LowerADCOffset(x.value as u8))
            }
            x if x.id == AnalogSectionId::UpperADCOffset as u8 => {
                Ok(AnalogSection::UpperADCOffset(x.value as u8))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

// render

impl From<AnalogSection> for Section {
    fn from(s: AnalogSection) -> Section {
        match s {
            AnalogSection::Enabled(value) => Section {
                id: AnalogSectionId::Enabled as u8,
                value: value as u16,
            },
            AnalogSection::InvertState(value) => Section {
                id: AnalogSectionId::InvertState as u8,
                value: value as u16,
            },
            AnalogSection::MessageType(value) => Section {
                id: AnalogSectionId::MessageType as u8,
                value: value.into(),
            },
            AnalogSection::MidiId(value) => Section {
                id: AnalogSectionId::MidiIdLSB as u8,
                value: value.into(),
            },
            AnalogSection::LowerCCLimit(value) => Section {
                id: AnalogSectionId::LowerCCLimitLSB as u8,
                value: value.into(),
            },
            AnalogSection::UpperCCLimit(value) => Section {
                id: AnalogSectionId::UpperCCLimitLSB as u8,
                value: value.into(),
            },
            AnalogSection::Channel(value) => Section {
                id: AnalogSectionId::Channel as u8,
                value: value.into(),
            },
            AnalogSection::LowerADCOffset(value) => Section {
                id: AnalogSectionId::LowerADCOffset as u8,
                value: value as u16,
            },
            AnalogSection::UpperADCOffset(value) => Section {
                id: AnalogSectionId::UpperADCOffset as u8,
                value: value as u16,
            },
        }
    }
}
