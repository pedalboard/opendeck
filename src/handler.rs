use crate::analog::handler::AnalogMessages;
use crate::button::handler::ButtonMessages;

use crate::ChannelOrAll;
use midi2::ux::u4;
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

pub struct ChannelMessages {
    channel_or_all: ChannelOrAll,
    nr_of_messages: usize,
    current_channel: i8,
    index: usize,
}

impl ChannelMessages {
    pub fn new_with_multiple_messages(channel_or_all: ChannelOrAll, nr_of_messages: usize) -> Self {
        Self {
            channel_or_all,
            nr_of_messages,
            current_channel: 0,
            index: 0,
        }
    }
    pub fn new(channel_or_all: ChannelOrAll) -> Self {
        Self {
            channel_or_all,
            nr_of_messages: 1,
            current_channel: 0,
            index: 0,
        }
    }
}

impl Iterator for ChannelMessages {
    type Item = (u4, usize);
    fn next(&mut self) -> Option<Self::Item> {
        match self.channel_or_all {
            ChannelOrAll::All => {
                if self.current_channel < 16 {
                    let r = Some((u4::new(self.current_channel as u8), self.index));
                    self.index += 1;
                    if self.index == self.nr_of_messages {
                        self.index = 0;
                        self.current_channel += 1;
                    }
                    r
                } else {
                    None
                }
            }
            ChannelOrAll::Channel(c) => {
                if self.index < self.nr_of_messages {
                    let r = Some((u4::new(c), self.index));
                    self.index += 1;
                    r
                } else {
                    None
                }
            }
            ChannelOrAll::None => {
                if self.index < self.nr_of_messages {
                    let r = Some((u4::new(0), self.index));
                    self.index += 1;
                    r
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use heapless::Vec;

    #[test]
    fn test_all_channels_single_message() {
        let all = ChannelMessages::new(ChannelOrAll::All).collect::<Vec<_, 16>>();
        let expected = (0..16).map(|i| (u4::new(i), 0)).collect::<Vec<_, 16>>();
        assert_eq!(all, expected);
    }
    #[test]
    fn test_all_channels_multipe_message() {
        let all = ChannelMessages::new_with_multiple_messages(ChannelOrAll::All, 3)
            .collect::<Vec<_, 48>>();
        let mut expected: Vec<_, 48> = Vec::new();
        for i in 0..16 {
            for j in 0..3 {
                expected.push((u4::new(i), j)).unwrap();
            }
        }
        assert_eq!(all, expected);
    }
    #[test]
    fn test_single_channel_single_message() {
        let single = ChannelMessages::new(ChannelOrAll::Channel(1)).collect::<Vec<_, 16>>();
        let mut expected: Vec<_, 48> = Vec::new();
        expected.push((u4::new(1), 0)).unwrap();
        assert_eq!(single, expected);
    }
    #[test]
    fn test_single_channel_multipe_message() {
        let all = ChannelMessages::new_with_multiple_messages(ChannelOrAll::Channel(2), 4)
            .collect::<Vec<_, 48>>();
        let mut expected: Vec<_, 48> = Vec::new();
        for j in 0..4 {
            expected.push((u4::new(2), j)).unwrap();
        }
        assert_eq!(all, expected);
    }
    #[test]
    fn test_no_channel_single_message() {
        let single = ChannelMessages::new(ChannelOrAll::None).collect::<Vec<_, 16>>();
        let mut expected: Vec<_, 48> = Vec::new();
        expected.push((u4::new(0), 0)).unwrap();
        assert_eq!(single, expected);
    }
    #[test]
    fn test_no() {
        let all = ChannelMessages::new_with_multiple_messages(ChannelOrAll::None, 2)
            .collect::<Vec<_, 48>>();
        let mut expected: Vec<_, 48> = Vec::new();
        for j in 0..2 {
            expected.push((u4::new(0), j)).unwrap();
        }
        assert_eq!(all, expected);
    }
}
