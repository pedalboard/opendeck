#![no_std]

// see https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration
use crate::{
    analog::AnalogSection, button::ButtonSection, config::FirmwareVersion, encoder::EncoderSection,
    global::GlobalSection, led::LedSection,
};
use heapless::Vec;

pub mod analog;
pub mod button;
pub mod config;
pub mod encoder;
pub mod global;
pub mod handler;
pub mod led;
pub mod parser;
pub mod renderer;

const SYSEX_START: u8 = 0xF0;
const SYSEX_END: u8 = 0xF7;
const M_ID_0: u8 = 0x00;
const M_ID_1: u8 = 0x53;
const M_ID_2: u8 = 0x43;

const BYTES_PER_VALUE: usize = 2;
const SPECIAL_REQ_MSG_SIZE: usize = 6 + 1 + 1; // extra byte for end
const STD_REQ_MIN_MSG_SIZE: usize = 10 + BYTES_PER_VALUE * 2 + 1;

// FIXME calculate value based on generic const
const PARAMS_PER_MESSAGE: usize = 32;

const MAX_MESSAGE_SIZE: usize = STD_REQ_MIN_MSG_SIZE + (PARAMS_PER_MESSAGE * BYTES_PER_VALUE);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpenDeckRequest {
    Special(SpecialRequest),
    Configuration(Wish, Amount, Block),
    ComponentInfo,
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
    fn seti(self) -> usize {
        self as usize - 1
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpecialRequest {
    Handshake = 0x01,
    ValueSize = 0x02,
    ValuesPerMessage = 0x03,
    FirmwareVersion = 0x56,
    HardwareUID = 0x42,
    FirmwareVersionAndHardwareUUID = 0x43,
    NrOfSupportedComponents = 0x4D,
    Reboot = 0x7F,
    BootloaderMode = 0x55,
    FactoryReset = 0x44,
    NrOfSupportedPresets = 0x50,
    BootloaderSupport = 0x51,
    Backup = 0x1B,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ValueSize {
    OneByte = 1,
    TwoBytes = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HardwareUid(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NrOfSupportedComponents {
    pub buttons: usize,
    pub encoders: usize,
    pub analog: usize,
    pub leds: usize,
    pub touchscreen_buttons: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpecialResponse {
    Handshake,
    ValueSize,
    ValuesPerMessage(u8),
    FirmwareVersion(FirmwareVersion),
    HardwareUID(HardwareUid),
    FirmwareVersionAndHardwareUUID(FirmwareVersion, HardwareUid),
    NrOfSupportedComponents(NrOfSupportedComponents),
    NrOfSupportedPresets(usize),
    BootloaderSupport(bool),
    Backup,
}

pub type NewValues = Vec<u16, PARAMS_PER_MESSAGE>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OpenDeckResponse {
    Special(SpecialResponse),
    Configuration(Wish, Amount, Block, NewValues),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelOrAll {
    All,
    Channel(u8),
    None,
}

impl Default for ChannelOrAll {
    fn default() -> Self {
        ChannelOrAll::Channel(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BlockId {
    Global,
    Button,
    Encoder,
    Analog,
    Led,
    Display,
    Touchscreen,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Block {
    Global(GlobalSection),
    Button(u16, ButtonSection),
    Encoder(u16, EncoderSection),
    Analog(u16, AnalogSection),
    Led(u16, LedSection),
    Display,
    Touchscreen,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Wish {
    Get,
    Set,
    Backup,
}

enum AmountId {
    Single,
    All,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Amount {
    Single,
    All(u8),
}

struct Section {
    id: u8,
    value: u16,
}
