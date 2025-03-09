use crate::analog::handler::AnalogMessages;
use crate::button::handler::ButtonMessages;

use midi2::{error::BufferOverflow, BytesMessage};

pub enum Messages<'a> {
    Button(ButtonMessages<'a>),
    Analog(AnalogMessages<'a>),
    None,
}

impl Messages<'_> {
    pub fn next<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<Option<BytesMessage<&'buf mut [u8]>>, BufferOverflow> {
        match self {
            Messages::Button(m) => m.next(buffer),
            Messages::Analog(m) => m.next(buffer),
            Messages::None => Ok(None),
        }
    }
}
