use crate::{
    Amount, AmountId, AnalogSection, AnalogSectionId, Block, BlockId, ButtonSection,
    ButtonSectionId, ButtonType, ByteOrder, ChannelOrAll, GlobalSection, GlobalSectionId,
    MessageStatus, MessageType, OpenDeckRequest, PresetIndex, Section, SpecialRequest, ValueSize,
    Wish, M_ID_0, M_ID_1, M_ID_2, SPECIAL_REQ_MSG_SIZE, SYSEX_END, SYSEX_START,
};
use midi_types::Value7;

impl TryFrom<u8> for SpecialRequest {
    type Error = OpenDeckParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == SpecialRequest::Handshake as u8 => Ok(SpecialRequest::Handshake),
            x if x == SpecialRequest::ValueSize as u8 => Ok(SpecialRequest::ValueSize),
            x if x == SpecialRequest::ValuesPerMessage as u8 => {
                Ok(SpecialRequest::ValuesPerMessage)
            }
            x if x == SpecialRequest::FirmwareVersion as u8 => Ok(SpecialRequest::FirmwareVersion),
            x if x == SpecialRequest::FirmwareVersionAndHardwareUUID as u8 => {
                Ok(SpecialRequest::FirmwareVersionAndHardwareUUID)
            }
            x if x == SpecialRequest::HardwareUID as u8 => Ok(SpecialRequest::HardwareUID),
            x if x == SpecialRequest::NrOfSupportedComponents as u8 => {
                Ok(SpecialRequest::NrOfSupportedComponents)
            }
            x if x == SpecialRequest::Reboot as u8 => Ok(SpecialRequest::Reboot),
            x if x == SpecialRequest::BootloaderMode as u8 => Ok(SpecialRequest::BootloaderMode),
            x if x == SpecialRequest::FactoryReset as u8 => Ok(SpecialRequest::FactoryReset),
            x if x == SpecialRequest::NrOfSupportedPresets as u8 => {
                Ok(SpecialRequest::NrOfSupportedPresets)
            }
            x if x == SpecialRequest::Backup as u8 => Ok(SpecialRequest::Backup),
            x if x == SpecialRequest::BootloaderSupport as u8 => {
                Ok(SpecialRequest::BootloaderSupport)
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::WishError)),
        }
    }
}

impl TryFrom<u8> for Wish {
    type Error = OpenDeckParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Wish::Get as u8 => Ok(Wish::Get),
            x if x == Wish::Set as u8 => Ok(Wish::Set),
            x if x == Wish::Backup as u8 => Ok(Wish::Backup),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::WishError)),
        }
    }
}

impl TryFrom<(u8, u8)> for Amount {
    type Error = OpenDeckParseError;
    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        match value {
            x if x.0 == AmountId::Single as u8 => Ok(Amount::Single),
            x if x.0 == AmountId::All as u8 => Ok(Amount::All(x.1)),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::AmountError)),
        }
    }
}

impl TryFrom<Section> for GlobalSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == GlobalSectionId::Midi as u8 => Ok(GlobalSection::Midi(x.value)),
            x if x.id == GlobalSectionId::Presets as u8 => Ok(GlobalSection::Presets(x.value)),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}
impl TryFrom<u16> for ButtonType {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == ButtonType::Momentary as u16 => Ok(ButtonType::Momentary),
            x if x == ButtonType::Latching as u16 => Ok(ButtonType::Latching),
            _ => Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError,
            )),
        }
    }
}

impl TryFrom<u16> for MessageType {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == MessageType::Notes as u16 => Ok(MessageType::Notes),
            x if x == MessageType::ProgramChange as u16 => Ok(MessageType::ProgramChange),
            x if x == MessageType::ControlChange as u16 => Ok(MessageType::ControlChange),
            x if x == MessageType::ControlChangeWithReset as u16 => {
                Ok(MessageType::ControlChangeWithReset)
            }
            x if x == MessageType::MMCStop as u16 => Ok(MessageType::MMCStop),
            x if x == MessageType::MMCPlay as u16 => Ok(MessageType::MMCPlay),
            x if x == MessageType::MMCRecord as u16 => Ok(MessageType::MMCRecord),
            x if x == MessageType::MMCPause as u16 => Ok(MessageType::MMCPause),
            x if x == MessageType::RealTimeClock as u16 => Ok(MessageType::RealTimeClock),
            x if x == MessageType::RealTimeStart as u16 => Ok(MessageType::RealTimeStart),
            x if x == MessageType::RealTimeContinue as u16 => Ok(MessageType::RealTimeContinue),
            x if x == MessageType::RealTimeStop as u16 => Ok(MessageType::RealTimeStop),
            x if x == MessageType::RealTimeActiveSensing as u16 => {
                Ok(MessageType::RealTimeActiveSensing)
            }
            x if x == MessageType::RealTimeSystemReset as u16 => {
                Ok(MessageType::RealTimeSystemReset)
            }
            x if x == MessageType::ProgramChangeDecr as u16 => Ok(MessageType::ProgramChangeDecr),
            x if x == MessageType::ProgramChangeIncr as u16 => Ok(MessageType::ProgramChangeIncr),
            x if x == MessageType::NoMessage as u16 => Ok(MessageType::NoMessage),
            x if x == MessageType::OpenDeckPresetChange as u16 => {
                Ok(MessageType::OpenDeckPresetChange)
            }
            x if x == MessageType::MultiValueIncNote as u16 => Ok(MessageType::MultiValueIncNote),
            x if x == MessageType::MultiValueDecNote as u16 => Ok(MessageType::MultiValueDecNote),
            x if x == MessageType::MultiValueIncCC as u16 => Ok(MessageType::MultiValueIncCC),
            x if x == MessageType::MultiValueDecCC as u16 => Ok(MessageType::MultiValueDecCC),
            x if x == MessageType::NoteOffOnly as u16 => Ok(MessageType::NoteOffOnly),
            x if x == MessageType::ControlChangeWithValue0 as u16 => {
                Ok(MessageType::ControlChangeWithValue0)
            }
            x if x == MessageType::ProgramChangeOffsetIncr as u16 => {
                Ok(MessageType::ProgramChangeOffsetIncr)
            }
            x if x == MessageType::ProgramChangeOffsetDecr as u16 => {
                Ok(MessageType::ProgramChangeOffsetDecr)
            }
            x if x == MessageType::BPMIncr as u16 => Ok(MessageType::BPMIncr),
            x if x == MessageType::BPMDecr as u16 => Ok(MessageType::BPMDecr),
            _ => Err(OpenDeckParseError::StatusError(
                MessageStatus::NewValueError,
            )),
        }
    }
}

impl TryFrom<Section> for ButtonSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == ButtonSectionId::MidiId as u8 => {
                Ok(ButtonSection::MidiId(Value7::from(x.value as u8)))
            }
            x if x.id == ButtonSectionId::MessageType as u8 => {
                let mt = MessageType::try_from(x.value)?;
                Ok(ButtonSection::MessageType(mt))
            }
            x if x.id == ButtonSectionId::Type as u8 => {
                let mt = ButtonType::try_from(x.value)?;
                Ok(ButtonSection::Type(mt))
            }
            x if x.id == ButtonSectionId::Value as u8 => {
                Ok(ButtonSection::Value(Value7::from(x.value as u8)))
            }
            x if x.id == ButtonSectionId::Channel as u8 => {
                Ok(ButtonSection::Channel(ChannelOrAll::from(x.value)))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

impl TryFrom<Section> for AnalogSection {
    type Error = OpenDeckParseError;
    fn try_from(value: Section) -> Result<Self, Self::Error> {
        match value {
            x if x.id == AnalogSectionId::Enabled as u8 => Ok(AnalogSection::Enabled(x.value)),
            x if x.id == AnalogSectionId::InvertState as u8 => {
                Ok(AnalogSection::InvertState(x.value))
            }
            x if x.id == AnalogSectionId::MessageType as u8 => {
                Ok(AnalogSection::MessageType(x.value))
            }
            x if x.id == AnalogSectionId::MidiIdLSB as u8 => Ok(AnalogSection::MidiIdLSB(x.value)),
            x if x.id == AnalogSectionId::MidiIdMSB as u8 => Ok(AnalogSection::MidiIdMSB(x.value)),
            x if x.id == AnalogSectionId::LowerCCLimitLSB as u8 => {
                Ok(AnalogSection::LowerCCLimitLSB(x.value))
            }
            x if x.id == AnalogSectionId::LowerCCLimitMSB as u8 => {
                Ok(AnalogSection::LowerCCLimitMSB(x.value))
            }
            x if x.id == AnalogSectionId::UpperCCLimitLSB as u8 => {
                Ok(AnalogSection::UpperCCLimitLSB(x.value))
            }
            x if x.id == AnalogSectionId::UpperCCLimitMSB as u8 => {
                Ok(AnalogSection::UpperCCLimitMSB(x.value))
            }
            x if x.id == AnalogSectionId::Channel as u8 => Ok(AnalogSection::Channel(x.value)),
            x if x.id == AnalogSectionId::LowerADCOffset as u8 => {
                Ok(AnalogSection::LowerADCOffset(x.value))
            }
            x if x.id == AnalogSectionId::UpperADCOffset as u8 => {
                Ok(AnalogSection::UpperADCOffset(x.value))
            }
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::SectionError)),
        }
    }
}

impl TryFrom<u16> for PresetIndex {
    type Error = OpenDeckParseError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            // FIXME support more preset values
            x if x == PresetIndex::Active as u16 => Ok(PresetIndex::Active),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::IndexError)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpenDeckParseError {
    BufferTooShort,
    NoSysex,
    WrongManufacturer,
    StatusError(MessageStatus),
}

pub struct OpenDeckParser {
    value_size: ValueSize,
}

impl OpenDeckParser {
    pub fn new(value_size: ValueSize) -> Self {
        OpenDeckParser { value_size }
    }

    pub fn parse(&self, buf: &[u8]) -> Result<OpenDeckRequest, OpenDeckParseError> {
        if buf.len() < 8 {
            return Err(OpenDeckParseError::BufferTooShort);
        }
        if ByteOrder::Start.get(buf) != SYSEX_START {
            return Err(OpenDeckParseError::NoSysex);
        }
        if buf[buf.len() - 1] != SYSEX_END {
            return Err(OpenDeckParseError::NoSysex);
        }

        if !(ByteOrder::Id1.get(buf) == M_ID_0
            && ByteOrder::Id2.get(buf) == M_ID_1
            && ByteOrder::Id3.get(buf) == M_ID_2)
        {
            return Err(OpenDeckParseError::WrongManufacturer);
        }

        if !ByteOrder::Status.get(buf) == MessageStatus::Request as u8 {
            return Err(OpenDeckParseError::StatusError(MessageStatus::StatusError));
        }
        if buf.len() == SPECIAL_REQ_MSG_SIZE {
            self.parse_special_request(buf)
        } else {
            self.parse_request(buf)
        }
    }

    pub fn parse_special_request(&self, buf: &[u8]) -> Result<OpenDeckRequest, OpenDeckParseError> {
        let special = SpecialRequest::try_from(ByteOrder::Wish.get(buf))?;
        Ok(OpenDeckRequest::Special(special))
    }

    pub fn parse_request(&self, buf: &[u8]) -> Result<OpenDeckRequest, OpenDeckParseError> {
        let wish = Wish::try_from(ByteOrder::Wish.get(buf))?;
        let amount = Amount::try_from((ByteOrder::Amount.get(buf), ByteOrder::Part.get(buf)))?;
        let block = self.parse_block(buf)?;
        Ok(OpenDeckRequest::Configuration(wish, amount, block))
    }
    pub fn parse_block(&self, buf: &[u8]) -> Result<Block, OpenDeckParseError> {
        let block_id = ByteOrder::Block.get(buf);
        let index = self.value_size.parse(buf, 0);
        let section = Section {
            id: ByteOrder::Section.get(buf),
            value: self.value_size.parse(buf, 1),
        };
        match block_id {
            x if x == BlockId::Global as u8 => {
                let section = GlobalSection::try_from(section)?;
                Ok(Block::Global(index, section))
            }
            x if x == BlockId::Button as u8 => {
                let bs = ButtonSection::try_from(section)?;
                Ok(Block::Button(index, bs))
            }
            x if x == BlockId::Encoder as u8 => Ok(Block::Encoder),
            x if x == BlockId::Analog as u8 => {
                let section = AnalogSection::try_from(section)?;
                Ok(Block::Analog(index, section))
            }
            x if x == BlockId::Led as u8 => Ok(Block::Led),
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::BlockError)),
        }
    }
}

impl ValueSize {
    fn parse(&self, buf: &[u8], index: usize) -> u16 {
        let start = ByteOrder::Index as usize;
        match self {
            ValueSize::OneByte => buf[start + index] as u16,
            ValueSize::TwoBytes => {
                let mut high = buf[start + index * 2];
                let mut low = buf[start + 1 + index * 2];

                if (high & 0x01) > 0 {
                    low |= 1 << 7;
                } else {
                    low &= !(1 << 7);
                }

                high >>= 1;

                let mut joined: u16;

                joined = high as u16;
                joined <<= 8;
                joined |= low as u16;
                joined
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_special_messages() {
        let p = OpenDeckParser {
            value_size: ValueSize::OneByte,
        };
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::Handshake))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x02, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::ValueSize))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x03, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::ValuesPerMessage))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x56, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::FirmwareVersion))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x42, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::HardwareUID))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x43, 0xF7]),
            Ok(OpenDeckRequest::Special(
                SpecialRequest::FirmwareVersionAndHardwareUUID
            ))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x4D, 0xF7]),
            Ok(OpenDeckRequest::Special(
                SpecialRequest::NrOfSupportedComponents
            ))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x7F, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::Reboot))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x44, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::FactoryReset))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x55, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::BootloaderMode))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x50, 0xF7]),
            Ok(OpenDeckRequest::Special(
                SpecialRequest::NrOfSupportedPresets
            ))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x51, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::BootloaderSupport))
        );
        assert_eq!(
            p.parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x1B, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::Backup))
        );
    }

    #[test]
    fn should_parse_configuration_messages() {
        let p = OpenDeckParser::new(ValueSize::OneByte);
        assert_eq!(
            p.parse(&[
                0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x05, 0x01, 0xF7
            ]),
            Ok(OpenDeckRequest::Configuration(
                Wish::Get,
                Amount::Single,
                Block::Analog(5, AnalogSection::MidiIdLSB(1)),
            ))
        );
        assert_eq!(
            p.parse(&[
                0xF0, 0x00, 0x53, 0x43, 0x00, 0x7F, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0xF7
            ]),
            Ok(OpenDeckRequest::Configuration(
                Wish::Get,
                Amount::All(0x7F),
                Block::Button(0, ButtonSection::MidiId(Value7::from(0))),
            ))
        );
    }
    #[test]
    fn should_split_u16() {
        let buf = &[
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x4E, 0x10, 0x00, 0x05,
        ];
        assert_eq!(10000, ValueSize::TwoBytes.parse(buf, 0));
        assert_eq!(5, ValueSize::TwoBytes.parse(buf, 1));
    }
}
