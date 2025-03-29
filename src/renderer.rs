pub struct OpenDeckRenderer<'buf> {
    buffer: &'buf mut [u8],
    value_size: ValueSize,
}

use crate::{
    Amount, AmountId, Block, BlockId, ByteOrder, ChannelOrAll, HardwareUid, MessageStatus,
    NrOfSupportedComponents, OpenDeckResponse, Section, SpecialRequest, SpecialResponse, ValueSize,
    MAX_MESSAGE_SIZE, M_ID_0, M_ID_1, M_ID_2,
};

use heapless::Vec;
use midi2::{error::BufferOverflow, prelude::*, sysex7::Sysex7};

type Buffer = Vec<u8, MAX_MESSAGE_SIZE>;

impl<'buf> OpenDeckRenderer<'buf> {
    pub fn new<'a: 'buf>(value_size: ValueSize, buffer: &'a mut [u8]) -> Self {
        OpenDeckRenderer { value_size, buffer }
    }

    pub fn render(
        self,
        res: OpenDeckResponse,
        status: MessageStatus,
    ) -> Result<Option<Sysex7<&'buf mut [u8]>>, BufferOverflow> {
        let mut buf = Vec::new();
        let mut m = Sysex7::try_new_with_buffer(self.buffer)?;

        buf.insert(ByteOrder::Id1.seti(), M_ID_0).unwrap();
        buf.insert(ByteOrder::Id2.seti(), M_ID_1).unwrap();
        buf.insert(ByteOrder::Id3.seti(), M_ID_2).unwrap();
        buf.insert(ByteOrder::Status.seti(), status as u8).unwrap();
        buf.insert(ByteOrder::Part.seti(), 0).unwrap();

        let wish = match res {
            OpenDeckResponse::Special(special) => match special {
                SpecialResponse::Handshake => SpecialRequest::Handshake as u8,
                SpecialResponse::ValueSize => {
                    let size = match self.value_size {
                        ValueSize::TwoBytes => 2,
                        ValueSize::OneByte => 1,
                    };
                    buf = self.value_size.push(size, buf);
                    SpecialRequest::ValueSize as u8
                }
                SpecialResponse::ValuesPerMessage(vpm) => {
                    buf = self.value_size.push(vpm as u16, buf);
                    SpecialRequest::ValuesPerMessage as u8
                }
                SpecialResponse::NrOfSupportedComponents(nr_of_comps) => {
                    buf = nr_of_comps.push(buf, &self.value_size);
                    SpecialRequest::NrOfSupportedComponents as u8
                }
                SpecialResponse::NrOfSupportedPresets(np) => {
                    buf = self.value_size.push(np as u16, buf);
                    SpecialRequest::NrOfSupportedPresets as u8
                }
                SpecialResponse::FirmwareVersion(v) => {
                    buf = v.push(buf, &self.value_size);
                    SpecialRequest::FirmwareVersion as u8
                }
                SpecialResponse::HardwareUID(uid) => {
                    buf = uid.push(buf, &self.value_size);
                    SpecialRequest::HardwareUID as u8
                }
                SpecialResponse::FirmwareVersionAndHardwareUUID(v, uid) => {
                    buf = v.push(buf, &self.value_size);
                    buf = uid.push(buf, &self.value_size);
                    SpecialRequest::FirmwareVersionAndHardwareUUID as u8
                }
                SpecialResponse::Backup => SpecialRequest::Backup as u8,
                SpecialResponse::BootloaderSupport(v) => {
                    buf = self.value_size.push(v as u16, buf);
                    SpecialRequest::BootloaderSupport as u8
                }
            },
            OpenDeckResponse::Configuration(wish, amount, block, new_values) => {
                buf = amount.push(buf);
                buf = block.push(buf, &self.value_size);
                for new_value in new_values.into_iter() {
                    buf = self.value_size.push(new_value, buf);
                }
                wish as u8
            }
        };

        buf.insert(ByteOrder::Wish.seti(), wish).unwrap();

        m.try_set_payload(buf.into_iter().map(u7::new))?;
        Ok(Some(m))
    }
}

impl ValueSize {
    fn push(&self, value: u16, mut buf: Buffer) -> Buffer {
        match self {
            ValueSize::OneByte => {
                // FIXME assert value < 128
                buf.push(value as u8).unwrap();
            }
            ValueSize::TwoBytes => {
                let mut new_high: u8 = ((value >> 8) & 0xFF) as u8;
                let mut new_low: u8 = (value & 0xFF) as u8;
                new_high = (new_high << 1) & 0x7F;

                if ((new_low >> 7) & 0x01) > 0 {
                    new_high |= 0x01;
                } else {
                    new_high &= !0x01;
                }

                new_low &= 0x7F;

                buf.push(new_high).unwrap();
                buf.push(new_low).unwrap();
            }
        }
        buf
    }
}

impl crate::config::FirmwareVersion {
    fn push(self, mut buf: Buffer, value_size: &ValueSize) -> Buffer {
        buf = value_size.push(self.major as u16, buf);
        buf = value_size.push(self.minor as u16, buf);
        value_size.push(self.revision as u16, buf)
    }
}

impl NrOfSupportedComponents {
    fn push(self, mut buf: Buffer, value_size: &ValueSize) -> Buffer {
        buf = value_size.push(self.buttons as u16, buf);
        buf = value_size.push(self.encoders as u16, buf);
        buf = value_size.push(self.analog as u16, buf);
        buf = value_size.push(self.leds as u16, buf);
        value_size.push(self.touchscreen_buttons as u16, buf)
    }
}

impl HardwareUid {
    fn push(self, mut buf: Buffer, value_size: &ValueSize) -> Buffer {
        buf = value_size.push(((self.0 >> 24) & 0xff) as u16, buf);
        buf = value_size.push(((self.0 >> 16) & 0xff) as u16, buf);
        buf = value_size.push(((self.0 >> 8) & 0xff) as u16, buf);
        value_size.push(((self.0) & 0xff) as u16, buf)
    }
}

impl Amount {
    fn push(self, mut buf: Buffer) -> Buffer {
        match self {
            Amount::Single => {
                buf.push(AmountId::Single as u8).unwrap();
            }
            Amount::All(part) => {
                buf[ByteOrder::Part.seti()] = part;
                buf.push(AmountId::All as u8).unwrap();
            }
        };
        buf
    }
}

impl From<ChannelOrAll> for u16 {
    fn from(value: ChannelOrAll) -> u16 {
        match value {
            ChannelOrAll::All => 17,
            ChannelOrAll::None => 0,
            ChannelOrAll::Channel(ch) => ch as u16 + 1,
        }
    }
}

impl Block {
    fn push(self, mut buf: Buffer, value_size: &ValueSize) -> Buffer {
        let (index, block_id, section) = match self {
            Block::Global(section) => {
                let result: (u16, Section) = section.into();
                (result.0, BlockId::Global, result.1)
            }
            Block::Button(i, section) => (i, BlockId::Button, section.into()),
            Block::Encoder(i, section) => (i, BlockId::Encoder, section.into()),
            Block::Analog(i, section) => (i, BlockId::Analog, section.into()),
            Block::Led(i, section) => (i, BlockId::Led, section.into()),
            Block::Display => (0, BlockId::Display, Section { id: 0, value: 0 }),
            Block::Touchscreen => (0, BlockId::Touchscreen, Section { id: 0, value: 0 }),
        };
        buf.push(block_id as u8).unwrap();
        buf.push(section.id).unwrap();
        buf = value_size.push(index, buf);
        buf = value_size.push(section.value, buf);
        buf
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        config::FirmwareVersion,
        global::{GlobalSection, PresetIndex},
        AnalogSection, HardwareUid, ValueSize, Wish,
    };

    #[test]
    fn should_render_special_messages_with_one_byte() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::Handshake),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_values_size() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValueSize),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x02, 0x01, 0xF7],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_values_per_message() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValuesPerMessage(0x20)),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x03, 0x20, 0xF7],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_firmware() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersion(FirmwareVersion {
                    major: 0x01,
                    minor: 0x02,
                    revision: 0x03,
                })),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x56, 0x01, 0x02, 0x03, 0xF7,
            ],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_hardware_uid() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::HardwareUID(HardwareUid(0x12345678))),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x42, 0x12, 0x34, 0x56, 0x78, 0xF7,
            ],
        );
    }

    #[test]
    fn should_render_special_messages_with_one_byte_firmware_hardware_uid() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    FirmwareVersion {
                        major: 0x03,
                        minor: 0x04,
                        revision: 0x05,
                    },
                    HardwareUid(0x12345678),
                )),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x43, 0x03, 0x04, 0x05, 0x12, 0x34, 0x56, 0x78,
                0xF7,
            ],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_nr_components() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedComponents(
                    crate::NrOfSupportedComponents {
                        buttons: 8,
                        encoders: 2,
                        analog: 2,
                        leds: 8,
                        touchscreen_buttons: 1,
                    },
                )),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x4D, 0x08, 0x02, 0x02, 0x08, 0x01, 0xF7,
            ],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_nr_presets() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedPresets(10)),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x50, 0x0A, 0xF7],
        );
    }
    #[test]
    fn should_render_special_messages_with_one_byte_bootloader_support() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::BootloaderSupport(true)),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x51, 0x01, 0xF7],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_handshake() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::Handshake),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_value_size() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValueSize),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x02, 0x00, 0x02, 0xF7],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_values_per_message() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValuesPerMessage(0x20)),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x03, 0x00, 0x20, 0xF7],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_fimrware() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersion(FirmwareVersion {
                    major: 0x01,
                    minor: 0x02,
                    revision: 0x03,
                })),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x56, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0xF7,
            ],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_hardware_uuid() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::HardwareUID(HardwareUid(0x12345678))),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x42, 0x00, 0x12, 0x00, 0x34, 0x00, 0x56, 0x00,
                0x78, 0xF7,
            ],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_firmware_and_hardware_uuid() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    FirmwareVersion {
                        major: 0x03,
                        minor: 0x04,
                        revision: 0x05,
                    },
                    HardwareUid(0x06070809),
                )),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x43, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00,
                0x06, 0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0xF7,
            ],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_nr_supported_components() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedComponents(
                    crate::NrOfSupportedComponents {
                        buttons: 8,
                        encoders: 2,
                        analog: 2,
                        leds: 8,
                        touchscreen_buttons: 1,
                    },
                )),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x4D, 0x00, 0x08, 0x00, 0x02, 0x00, 0x02, 0x00,
                0x08, 0x00, 0x01, 0xF7,
            ],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_nr_presets() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedPresets(10)),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x50, 0x00, 0x0A, 0xF7],
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes_bootloader_support() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::BootloaderSupport(true)),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x51, 0x00, 0x01, 0xF7],
        );
    }
    #[test]
    fn should_render_configuration_messages_with_two_bytes() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };
        assert_sysex(
            renderer.render(
                OpenDeckResponse::Configuration(
                    Wish::Get,
                    Amount::Single,
                    Block::Analog(5, AnalogSection::MidiId(u16::MIN)),
                    Vec::from_slice(&[5]).unwrap(),
                ),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00,
                0x00, 0x05, 0xF7,
            ],
        );
    }
    #[test]
    fn should_render_configuration_messages_global() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Configuration(
                    Wish::Get,
                    Amount::Single,
                    Block::Global(GlobalSection::Presets(PresetIndex::Active, 0x00)),
                    Vec::new(),
                ),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
                0xF7,
            ],
        );
    }
    #[test]
    fn should_render_configuration_messages_analog() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
            buffer: &mut [0; MAX_MESSAGE_SIZE],
        };

        assert_sysex(
            renderer.render(
                OpenDeckResponse::Configuration(
                    Wish::Get,
                    Amount::All(0x00),
                    Block::Analog(0, AnalogSection::MidiId(u16::MIN)),
                    Vec::from_slice(&[5, 6, 7, 8]).unwrap(),
                ),
                MessageStatus::Response,
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x01, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x05, 0x00, 0x06, 0x00, 0x07, 0x00, 0x08, 0xF7,
            ],
        );
    }

    fn assert_sysex(act: Result<Option<Sysex7<&mut [u8]>>, BufferOverflow>, exp: &[u8]) {
        assert_eq!(act.unwrap().unwrap().data(), exp)
    }

    #[test]
    fn should_render_u16() {
        let buf = Vec::new();
        assert_eq!(ValueSize::TwoBytes.push(10000, buf), &[0x4E, 0x10]);
    }
}
