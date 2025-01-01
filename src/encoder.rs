use crate::{parser::OpenDeckParseError, ChannelOrAll, MessageStatus, Section};
use int_enum::IntEnum;
use midi_types::{Value14, Value7};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Encoder {
    enabled: bool,
    invert_state: bool,
    message_type: EncoderMessageType,
    midi_id: Value14,
    channel: ChannelOrAll,
    pulses_per_step: u8,
    accelleration: Accelleration,
    remote_sync: bool,
    upper_limit: Value14,
    lower_limit: Value14,
    repeated_value: Value14,
    second_midi_id: Value14,
}

impl Encoder {
    pub fn new(midi_id: Value14) -> Self {
        Encoder {
            enabled: true,
            invert_state: false,
            message_type: EncoderMessageType::default(),
            channel: ChannelOrAll::default(),
            pulses_per_step: 2,
            midi_id,
            accelleration: Accelleration::None,
            remote_sync: false,
            lower_limit: Value14::from(u16::MIN),
            upper_limit: Value14::from(u16::MIN),
            second_midi_id: Value14::from(u16::MIN),
            repeated_value: Value14::from(u16::MIN),
        }
    }
    pub fn set(&mut self, section: &EncoderSection) {
        match section {
            EncoderSection::MessageType(v) => self.message_type = *v,
            EncoderSection::Channel(v) => self.channel = v.clone(),
            EncoderSection::Enabled(v) => self.enabled = *v,
            EncoderSection::MidiIdLSB(v) => self.midi_id = *v,
            EncoderSection::InvertState(v) => self.invert_state = *v,
            EncoderSection::PulsesPerStep(v) => self.pulses_per_step = *v,
            EncoderSection::RemoteSync(v) => self.remote_sync = *v,
            EncoderSection::Accelleration(v) => self.accelleration = *v,
            EncoderSection::LowerLimit(v) => self.lower_limit = *v,
            EncoderSection::UpperLimit(v) => self.upper_limit = *v,
            EncoderSection::SecondMidiId(v) => self.second_midi_id = *v,
            EncoderSection::RepeatedValue(v) => self.repeated_value = *v,
            EncoderSection::MidiIdMSB(_) => {}
        }
    }
    pub fn get(&self, section: &EncoderSection) -> u16 {
        match section {
            EncoderSection::MessageType(_) => self.message_type as u16,
            EncoderSection::Channel(_) => self.channel.clone().into(),
            EncoderSection::Enabled(_) => self.enabled as u16,
            EncoderSection::MidiIdLSB(_) => self.midi_id.into(),
            EncoderSection::InvertState(_) => self.invert_state as u16,
            EncoderSection::PulsesPerStep(_) => self.pulses_per_step as u16,
            EncoderSection::RemoteSync(_) => self.remote_sync as u16,
            EncoderSection::Accelleration(_) => self.accelleration as u16,
            EncoderSection::LowerLimit(_) => self.lower_limit.into(),
            EncoderSection::UpperLimit(_) => self.upper_limit.into(),
            EncoderSection::SecondMidiId(_) => self.second_midi_id.into(),
            EncoderSection::RepeatedValue(_) => self.repeated_value.into(),
            EncoderSection::MidiIdMSB(_) => 0x00,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum EncoderMessageType {
    #[default]
    ControlChange7Fh01h = 0x0,
    ControlChange3Fh41h = 0x1,
    ProgramChange = 0x2,
    ControlChange = 0x3,
    PresetChange = 0x4,
    PitchBend = 0x5,
    NRPN7 = 0x6,
    NRPN8 = 0x7,
    ControlChange14bit = 0x8,
    ControlChange41h01h = 0x9,
    BPM = 0xA,
    SingleNoteWithVariableValue = 0xB,
    SingleNoteWithFixedValueBothDirections = 0xC,
    SingleNoteWithFixedValueOneDirection0OtherDirection = 0xD,
    TwoNoteWithFixedValueBothDirections = 0xE,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntEnum, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
pub enum Accelleration {
    #[default]
    None = 0,
    Slow = 1,
    Medium = 2,
    Fast = 3,
}

#[allow(dead_code)]
enum EncoderSectionId {
    Enabled,
    InvertState,
    MessageType,
    MidiIdLSB,
    Channel,
    PulsesPerStep,
    Accelleration,
    MidiIdMSB, // only used in 1 byte protocol
    RemoteSync,
    LowerLimit,
    UpperLimit,
    RepeatedValue,
    SecondMidiId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EncoderSection {
    Enabled(bool),
    InvertState(bool),
    MessageType(EncoderMessageType),
    MidiIdLSB(Value14),
    Channel(ChannelOrAll),
    PulsesPerStep(u8),
    Accelleration(Accelleration),
    MidiIdMSB(Value7),
    RemoteSync(bool),
    LowerLimit(Value14),
    UpperLimit(Value14),
    RepeatedValue(Value14),
    SecondMidiId(Value14),
}

impl From<EncoderSection> for Section {
    fn from(s: EncoderSection) -> Section {
        match s {
            EncoderSection::Enabled(value) => Section {
                id: EncoderSectionId::Enabled as u8,
                value: value as u16,
            },
            EncoderSection::RemoteSync(value) => Section {
                id: EncoderSectionId::RemoteSync as u8,
                value: value as u16,
            },
            EncoderSection::InvertState(value) => Section {
                id: EncoderSectionId::InvertState as u8,
                value: value as u16,
            },
            EncoderSection::Channel(value) => Section {
                id: EncoderSectionId::Channel as u8,
                value: value.into(),
            },
            EncoderSection::MessageType(value) => Section {
                id: EncoderSectionId::MessageType as u8,
                value: value as u16,
            },
            EncoderSection::Accelleration(value) => Section {
                id: EncoderSectionId::Accelleration as u8,
                value: value as u16,
            },
            EncoderSection::PulsesPerStep(value) => Section {
                id: EncoderSectionId::PulsesPerStep as u8,
                value: value as u16,
            },
            EncoderSection::MidiIdLSB(v) => Section {
                id: EncoderSectionId::MidiIdLSB as u8,
                value: v.into(),
            },
            EncoderSection::MidiIdMSB(v) => {
                let value: u8 = v.into();
                Section {
                    id: EncoderSectionId::MidiIdMSB as u8,
                    value: value as u16,
                }
            }
            EncoderSection::LowerLimit(v) => Section {
                id: EncoderSectionId::LowerLimit as u8,
                value: v.into(),
            },
            EncoderSection::UpperLimit(v) => Section {
                id: EncoderSectionId::UpperLimit as u8,
                value: v.into(),
            },
            EncoderSection::RepeatedValue(v) => Section {
                id: EncoderSectionId::RepeatedValue as u8,
                value: v.into(),
            },
            EncoderSection::SecondMidiId(v) => Section {
                id: EncoderSectionId::SecondMidiId as u8,
                value: v.into(),
            },
        }
    }
}

impl TryFrom<Section> for EncoderSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == EncoderSectionId::InvertState as u8 => {
                Ok(EncoderSection::InvertState(x.value > 0))
            }
            x if x.id == EncoderSectionId::RemoteSync as u8 => {
                Ok(EncoderSection::RemoteSync(x.value > 0))
            }
            x if x.id == EncoderSectionId::Enabled as u8 => {
                Ok(EncoderSection::Enabled(x.value > 0))
            }
            x if x.id == EncoderSectionId::MessageType as u8 => {
                if let Ok(mt) = EncoderMessageType::try_from(x.value) {
                    Ok(EncoderSection::MessageType(mt))
                } else {
                    Err(OpenDeckParseError::StatusError(
                        MessageStatus::NewValueError,
                    ))
                }
            }
            x if x.id == EncoderSectionId::Channel as u8 => {
                Ok(EncoderSection::Channel(ChannelOrAll::from(x.value)))
            }
            x if x.id == EncoderSectionId::Accelleration as u8 => {
                if let Ok(a) = Accelleration::try_from(x.value) {
                    Ok(EncoderSection::Accelleration(a))
                } else {
                    Err(OpenDeckParseError::StatusError(
                        MessageStatus::NewValueError,
                    ))
                }
            }
            x if x.id == EncoderSectionId::PulsesPerStep as u8 => {
                Ok(EncoderSection::PulsesPerStep(x.value as u8))
            }
            x if x.id == EncoderSectionId::MidiIdLSB as u8 => {
                Ok(EncoderSection::MidiIdLSB(Value14::from(x.value)))
            }
            x if x.id == EncoderSectionId::MidiIdMSB as u8 => {
                Ok(EncoderSection::MidiIdMSB(Value7::from(x.value as u8)))
            }
            x if x.id == EncoderSectionId::LowerLimit as u8 => {
                Ok(EncoderSection::LowerLimit(Value14::from(x.value)))
            }
            x if x.id == EncoderSectionId::UpperLimit as u8 => {
                Ok(EncoderSection::UpperLimit(Value14::from(x.value)))
            }
            x if x.id == EncoderSectionId::RepeatedValue as u8 => {
                Ok(EncoderSection::RepeatedValue(Value14::from(x.value)))
            }
            x if x.id == EncoderSectionId::SecondMidiId as u8 => {
                Ok(EncoderSection::SecondMidiId(Value14::from(x.value)))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}
