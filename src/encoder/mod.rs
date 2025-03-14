use crate::ChannelOrAll;
use int_enum::IntEnum;

pub mod handler;
pub mod parser;
pub mod renderer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Encoder {
    enabled: bool,
    inverted: bool,
    message_type: EncoderMessageType,
    midi_id: u16,
    channel: ChannelOrAll,
    pulses_per_step: u8,
    accelleration: Accelleration,
    remote_sync: bool,
    upper_limit: u16,
    lower_limit: u16,
    value: u16,
    second_midi_id: u16,
}

impl Encoder {
    pub fn new(midi_id: u16) -> Self {
        Encoder {
            enabled: true,
            inverted: false,
            message_type: EncoderMessageType::default(),
            channel: ChannelOrAll::default(),
            pulses_per_step: 2,
            midi_id,
            accelleration: Accelleration::None,
            remote_sync: false,
            lower_limit: u16::MIN,
            upper_limit: u16::MIN,
            second_midi_id: u16::MIN,
            value: u16::MIN,
        }
    }
    pub fn set(&mut self, section: &EncoderSection) {
        match section {
            EncoderSection::MessageType(v) => self.message_type = *v,
            EncoderSection::Channel(v) => self.channel = *v,
            EncoderSection::Enabled(v) => self.enabled = *v,
            EncoderSection::MidiIdLSB(v) => self.midi_id = *v,
            EncoderSection::Inverted(v) => self.inverted = *v,
            EncoderSection::PulsesPerStep(v) => self.pulses_per_step = *v,
            EncoderSection::RemoteSync(v) => self.remote_sync = *v,
            EncoderSection::Accelleration(v) => self.accelleration = *v,
            EncoderSection::LowerLimit(v) => self.lower_limit = *v,
            EncoderSection::UpperLimit(v) => self.upper_limit = *v,
            EncoderSection::SecondMidiId(v) => self.second_midi_id = *v,
            EncoderSection::RepeatedValue(v) => self.value = *v,
            EncoderSection::MidiIdMSB(_) => {}
        }
    }
    pub fn get(&self, section: &EncoderSection) -> u16 {
        match section {
            EncoderSection::MessageType(_) => self.message_type.into(),
            EncoderSection::Channel(_) => self.channel.into(),
            EncoderSection::Enabled(_) => self.enabled.into(),
            EncoderSection::MidiIdLSB(_) => self.midi_id,
            EncoderSection::Inverted(_) => self.inverted.into(),
            EncoderSection::PulsesPerStep(_) => self.pulses_per_step.into(),
            EncoderSection::RemoteSync(_) => self.remote_sync.into(),
            EncoderSection::Accelleration(_) => self.accelleration.into(),
            EncoderSection::LowerLimit(_) => self.lower_limit,
            EncoderSection::UpperLimit(_) => self.upper_limit,
            EncoderSection::SecondMidiId(_) => self.second_midi_id,
            EncoderSection::RepeatedValue(_) => self.value,
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
    NRPN14 = 0x7,
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

#[derive(IntEnum)]
#[repr(u8)]
enum EncoderSectionId {
    Enabled = 0x0,
    InvertState = 0x1,
    MessageType = 0x2,
    MidiIdLSB = 0x3,
    Channel = 0x4,
    PulsesPerStep = 0x5,
    Accelleration = 0x6,
    MidiIdMSB = 0x7, // only used in 1 byte protocol
    RemoteSync = 0x8,
    LowerLimit = 0x9,
    UpperLimit = 0xA,
    RepeatedValue = 0xB,
    SecondMidiId = 0xC,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EncoderSection {
    Enabled(bool),
    Inverted(bool),
    MessageType(EncoderMessageType),
    MidiIdLSB(u16),
    Channel(ChannelOrAll),
    PulsesPerStep(u8),
    Accelleration(Accelleration),
    MidiIdMSB(u8),
    RemoteSync(bool),
    LowerLimit(u16),
    UpperLimit(u16),
    RepeatedValue(u16),
    SecondMidiId(u16),
}
