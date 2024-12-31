use crate::{parser::OpenDeckParseError, ChannelOrAll, MessageStatus, Section};
use midi_types::{Value14, Value7};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EncoderMessageType {
    #[default]
    ControlChange7Fh01h,
    ControlChange3Fh41h,
    ProgramChange,
    ControlChange,
    PresetChange,
    PitchBend,
    NRPN7,
    NRPN8,
    ControlChange14bit,
    ControlChange41h01h,
    BPM,
    Note,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Accelleration {
    #[default]
    None,
    Slow,
    Medium,
    Fast,
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
    MidiIdMSB,
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

impl TryFrom<u16> for Accelleration {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            // FIXME support more preset values
            x if x == Accelleration::None as u16 => Ok(Accelleration::None),
            x if x == Accelleration::Slow as u16 => Ok(Accelleration::Slow),
            x if x == Accelleration::Medium as u16 => Ok(Accelleration::Medium),
            x if x == Accelleration::Fast as u16 => Ok(Accelleration::Fast),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::IndexError)),
        }
    }
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

impl TryFrom<u16> for EncoderMessageType {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == EncoderMessageType::ControlChange7Fh01h as u16 => {
                Ok(EncoderMessageType::ControlChange7Fh01h)
            }
            x if x == EncoderMessageType::ControlChange3Fh41h as u16 => {
                Ok(EncoderMessageType::ControlChange3Fh41h)
            }
            x if x == EncoderMessageType::ProgramChange as u16 => {
                Ok(EncoderMessageType::ProgramChange)
            }
            x if x == EncoderMessageType::ControlChange as u16 => {
                Ok(EncoderMessageType::ControlChange)
            }
            x if x == EncoderMessageType::PresetChange as u16 => {
                Ok(EncoderMessageType::PresetChange)
            }
            x if x == EncoderMessageType::PitchBend as u16 => Ok(EncoderMessageType::PitchBend),
            x if x == EncoderMessageType::NRPN7 as u16 => Ok(EncoderMessageType::NRPN7),
            x if x == EncoderMessageType::NRPN8 as u16 => Ok(EncoderMessageType::NRPN8),
            x if x == EncoderMessageType::ControlChange14bit as u16 => {
                Ok(EncoderMessageType::ControlChange14bit)
            }
            x if x == EncoderMessageType::ControlChange41h01h as u16 => {
                Ok(EncoderMessageType::ControlChange41h01h)
            }
            x if x == EncoderMessageType::BPM as u16 => Ok(EncoderMessageType::BPM),
            x if x == EncoderMessageType::Note as u16 => Ok(EncoderMessageType::Note),
            _ => Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError,
            )),
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
                let mt = EncoderMessageType::try_from(x.value)?;
                Ok(EncoderSection::MessageType(mt))
            }
            x if x.id == EncoderSectionId::Channel as u8 => {
                Ok(EncoderSection::Channel(ChannelOrAll::from(x.value)))
            }
            x if x.id == EncoderSectionId::Accelleration as u8 => {
                let ac = Accelleration::try_from(x.value)?;
                Ok(EncoderSection::Accelleration(ac))
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
