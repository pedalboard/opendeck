use crate::ChannelOrAll;
use int_enum::IntEnum;

pub mod handler;
pub mod parser;
pub mod renderer;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Analog {
    enabled: bool,
    inverted: bool,
    message_type: AnalogMessageType,
    midi_id: u16,
    upper_limit: u16,
    lower_limit: u16,
    channel: ChannelOrAll,
    lower_adc_offset: u8,
    upper_adc_offset: u8,
}

impl Analog {
    pub fn new(midi_id: u16) -> Self {
        Analog {
            enabled: false,
            inverted: false,
            message_type: AnalogMessageType::default(),
            channel: ChannelOrAll::default(),
            midi_id,
            lower_limit: u16::MIN,
            upper_limit: 0x7F,
            lower_adc_offset: 0,
            upper_adc_offset: 0,
        }
    }
    pub fn set(&mut self, section: AnalogSection) {
        match section {
            AnalogSection::MessageType(v) => self.message_type = v,
            AnalogSection::Channel(v) => self.channel = v,
            AnalogSection::Enabled(v) => self.enabled = v,
            AnalogSection::MidiId(v) => self.midi_id = v,
            AnalogSection::Inverted(v) => self.inverted = v,
            AnalogSection::LowerCCLimit(v) => self.lower_limit = v,
            AnalogSection::UpperCCLimit(v) => self.upper_limit = v,
            AnalogSection::LowerADCOffset(v) => self.lower_adc_offset = v,
            AnalogSection::UpperADCOffset(v) => self.upper_adc_offset = v,
        }
    }
    pub fn get(&self, section: AnalogSection) -> u16 {
        match section {
            AnalogSection::MessageType(_) => self.message_type as u16,
            AnalogSection::Channel(_) => self.channel.into(),
            AnalogSection::Enabled(_) => self.enabled as u16,
            AnalogSection::MidiId(_) => self.midi_id,
            AnalogSection::Inverted(_) => self.inverted as u16,
            AnalogSection::LowerCCLimit(_) => self.lower_limit,
            AnalogSection::UpperCCLimit(_) => self.upper_limit,
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
    NRPN14 = 5,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnalogSection {
    Enabled(bool),
    Inverted(bool),
    MessageType(AnalogMessageType),
    MidiId(u16),
    LowerCCLimit(u16),
    UpperCCLimit(u16),
    Channel(ChannelOrAll),
    LowerADCOffset(u8),
    UpperADCOffset(u8),
}
