use crate::analog::handler::AnalogMessages;
use crate::button::handler::ButtonMessages;

use crate::ChannelOrAll;
use midi2::ux::u4;
use midi2::{error::BufferOverflow, BytesMessage};

// FIXME move this to top level

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

pub struct Channels {
    channel_or_all: ChannelOrAll,
    current: i8,
}

impl Channels {
    pub fn new(channel_or_all: ChannelOrAll) -> Self {
        Self {
            channel_or_all,
            current: -1,
        }
    }
}

impl Iterator for Channels {
    type Item = u4;
    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        match self.channel_or_all {
            ChannelOrAll::All => {
                if self.current < 16 {
                    Some(u4::new(self.current as u8))
                } else {
                    None
                }
            }
            ChannelOrAll::Channel(c) => {
                if self.current == 0 {
                    Some(u4::new(c))
                } else {
                    None
                }
            }
            ChannelOrAll::None => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use heapless::Vec;

    #[test]
    fn test_all() {
        let all = Channels::new(ChannelOrAll::All).collect::<Vec<_, 16>>();
        let expected = (0..16).map(|i| u4::new(i)).collect::<Vec<_, 16>>();
        assert_eq!(all, expected);
    }
}
