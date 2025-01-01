use crate::ChannelOrAll;
use int_enum::IntEnum;
use midi_types::Value14;

pub mod parser;
pub mod renderer;

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
            AnalogSection::Channel(v) => self.channel = *v,
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
            AnalogSection::Channel(_) => self.channel.into(),
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

#[derive(IntEnum)]
#[repr(u8)]
enum AnalogSectionId {
    Enabled = 0x0,
    InvertState = 0x1,
    MessageType = 0x2,
    MidiIdLSB = 0x3,
    // MidiIdMSB = 0x4, // only used in 1 byte protocol
    LowerCCLimitLSB = 0x5,
    //LowerCCLimitMSB = 0x6, // only used in 1 byte protocol
    UpperCCLimitLSB = 0x7,
    // UpperCCLimitMSB = 0x8, // only used in 1 byte protocol
    Channel = 0x9,
    LowerADCOffset = 0xA,
    UpperADCOffset = 0xB,
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
