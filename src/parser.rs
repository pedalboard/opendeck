use crate::{
    ByteOrder, MessageStatus, OpenDeckRequest, SpecialRequest, M_ID_0, M_ID_1, M_ID_2,
    SPECIAL_REQ_MSG_SIZE, SYSEX_END, SYSEX_START,
};

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
            _ => Err(OpenDeckParseError::StatusError(MessageStatus::StatusError)),
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

pub struct OpenDeckParser {}

impl OpenDeckParser {
    pub fn parse(buf: &[u8]) -> Result<OpenDeckRequest, OpenDeckParseError> {
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
            return OpenDeckParser::parse_special_request(buf);
        }

        Err(OpenDeckParseError::BufferTooShort)
    }

    pub fn parse_special_request(buf: &[u8]) -> Result<OpenDeckRequest, OpenDeckParseError> {
        let special = SpecialRequest::try_from(ByteOrder::Wish.get(buf))?;
        Ok(OpenDeckRequest::Special(special))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_special_messages() {
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::Handshake))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x02, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::ValueSize))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x03, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::ValuesPerMessage))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x56, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::FirmwareVersion))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x42, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::HardwareUID))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x43, 0xF7]),
            Ok(OpenDeckRequest::Special(
                SpecialRequest::FirmwareVersionAndHardwareUUID
            ))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x4D, 0xF7]),
            Ok(OpenDeckRequest::Special(
                SpecialRequest::NrOfSupportedComponents
            ))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x7F, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::Reboot))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x44, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::FactoryReset))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x55, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::BootloaderMode))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x50, 0xF7]),
            Ok(OpenDeckRequest::Special(
                SpecialRequest::NrOfSupportedPresets
            ))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x51, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::BootloaderSupport))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x1B, 0xF7]),
            Ok(OpenDeckRequest::Special(SpecialRequest::Backup))
        );
    }
}
