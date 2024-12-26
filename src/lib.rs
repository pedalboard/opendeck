#![no_std]

// see https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration

const SYSEX_START: u8 = 0xF0;
const SYSEX_END: u8 = 0xF7;
const M_ID_0: u8 = 0x00;
const M_ID_1: u8 = 0x53;
const M_ID_2: u8 = 0x43;

const PARAMS_PER_MESSAGE: usize = 32;
const BYTES_PER_VALUE: usize = 2;
const SPECIAL_REQ_MSG_SIZE: usize = 6 + 1 + 1; // extra byte for end
const STD_REQ_MIN_MSG_SIZE: usize = 10 + BYTES_PER_VALUE * 2 + 1;
const MAX_MESSAGE_SIZE: usize = STD_REQ_MIN_MSG_SIZE + (PARAMS_PER_MESSAGE * BYTES_PER_VALUE);

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MessageStatus {
    Request = 0x00,
    // Request was valid if response sets MESSAGE_STATUS to ACK (1).
    Response = 0x01,
    // This error happens when MESSAGE_STATUS isn't REQUEST (0) in request.
    StatusError = 0x02,
    // This error is returned when request is correct, but handshake request hasn't been sent to board (or SysEx connection has been closed).
    HandshakeError = 0x03,
    // This error is returned when WISH is anything other than GET, SET or BACKUP.
    WishError = 0x04,
    // This error is returned when AMOUNT is anything other than SINGLE or ALL.
    AmountError = 0x05,
    // This error is returned when BLOCK byte is incorrect.
    BlockError = 0x06,
    // This error is returned when SECTION byte is incorrect.
    SectionError = 0x07,
    // This error is returned when message part is incorrect.
    PartError = 0x08,
    // This error is returned when wanted parameter is incorrect.
    IndexError = 0x09,
    // This error is returned when NEW_VALUE is incorrect.
    NewValueError = 0x0A,
    // This error is returned when request is too short.
    MessageLengthError = 0x0B,
    // This error is returned when writing new value to board has failed. This can happen if EEPROM on board is damaged.
    WriteError = 0x0C,
    // This error is returned when the requested parameter isn't supported on the board.
    NotSupportedError = 0x0D,
    // This error is returned when the reading of requested index fails.
    ReadError = 0x0E,
    //
    UARTAllocationError = 0x80,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpenDeckMsg {
    Special(SpecialRequest),
    Configuration,
    ComponentInfo,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpenDeckParseError {
    BufferTooShort,
    NoSysex,
    WrongManufacturer,
    StatusError(MessageStatus),
}

enum ByteOrder {
    Start,
    Id1,
    Id2,
    Id3,
    Status,
    Part,
    Wish,
    Amount,
    Block,
    Section,
    Index,
}

impl ByteOrder {
    fn get(self, buf: &[u8]) -> u8 {
        buf[self as usize]
    }
}

pub struct OpenDeckParser {}

impl OpenDeckParser {
    pub fn parse(buf: &[u8]) -> Result<OpenDeckMsg, OpenDeckParseError> {
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

    pub fn parse_special_request(buf: &[u8]) -> Result<OpenDeckMsg, OpenDeckParseError> {
        let special = SpecialRequest::try_from(ByteOrder::Wish.get(buf))?;
        Ok(OpenDeckMsg::Special(special))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpecialRequest {
    Handshake = 0x01,
    ValueSize = 0x02,
    ValuesPerMessage = 0x03,
    FirmwareVersion = 0x56,
    HardwareUUID = 0x42,
    FirmwareVersionAndHardwareUUID = 0x43,
    NrOfSupportedComponents = 0x4D,
    Reboot = 0x7F,
    BootloaderMode = 0x55,
    FactoryReset = 0x44,
    NrOfSupportedPresets = 0x50,
    BootloaderSupport = 0x51,
    Backup = 0x1B,
}

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
            x if x == SpecialRequest::HardwareUUID as u8 => Ok(SpecialRequest::HardwareUUID),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_special_messages() {
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::Handshake))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x02, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::ValueSize))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x03, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::ValuesPerMessage))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x56, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::FirmwareVersion))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x42, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::HardwareUUID))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x43, 0xF7]),
            Ok(OpenDeckMsg::Special(
                SpecialRequest::FirmwareVersionAndHardwareUUID
            ))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x4D, 0xF7]),
            Ok(OpenDeckMsg::Special(
                SpecialRequest::NrOfSupportedComponents
            ))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x7F, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::Reboot))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x44, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::FactoryReset))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x55, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::BootloaderMode))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x50, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::NrOfSupportedPresets))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x51, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::BootloaderSupport))
        );
        assert_eq!(
            OpenDeckParser::parse(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x1B, 0xF7]),
            Ok(OpenDeckMsg::Special(SpecialRequest::Backup))
        );
    }
}
