pub struct OpenDeckRenderer {
    value_size: ValueSize,
}

use crate::{
    ByteOrder, FirmwareVersion, HardwareUid, MessageStatus, NrOfSupportedComponents,
    OpenDeckResponse, SpecialRequest, SpecialResponse, ValueSize, MAX_MESSAGE_SIZE, M_ID_0, M_ID_1,
    M_ID_2, SYSEX_END, SYSEX_START,
};
use heapless::Vec;

pub type Buffer = Vec<u8, MAX_MESSAGE_SIZE>;

impl OpenDeckRenderer {
    pub fn new(value_size: ValueSize) -> Self {
        OpenDeckRenderer { value_size }
    }

    pub fn render(&self, res: OpenDeckResponse, status: MessageStatus) -> Buffer {
        let mut buf = Vec::new();
        buf.insert(ByteOrder::Start as usize, SYSEX_START).unwrap();
        buf.insert(ByteOrder::Id1 as usize, M_ID_0).unwrap();
        buf.insert(ByteOrder::Id2 as usize, M_ID_1).unwrap();
        buf.insert(ByteOrder::Id3 as usize, M_ID_2).unwrap();
        buf.insert(ByteOrder::Status as usize, status as u8)
            .unwrap();
        buf.insert(ByteOrder::Part as usize, 0).unwrap();

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
        };

        buf.insert(ByteOrder::Wish as usize, wish).unwrap();
        buf.push(SYSEX_END).unwrap();
        buf
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

impl FirmwareVersion {
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{FirmwareVersion, HardwareUid, ValueSize};

    #[test]
    fn should_render_special_messages_with_one_byte() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::OneByte,
        };
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::Handshake),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValueSize),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x02, 0x01, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValuesPerMessage(0x20)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x03, 0x20, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersion(FirmwareVersion {
                    major: 0x01,
                    minor: 0x02,
                    revision: 0x03,
                })),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x56, 0x01, 0x02, 0x03, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::HardwareUID(HardwareUid(0x12345678))),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x42, 0x12, 0x34, 0x56, 0x78, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    FirmwareVersion {
                        major: 0x03,
                        minor: 0x04,
                        revision: 0x05,
                    },
                    HardwareUid(0xA2B4C6D8)
                )),
                MessageStatus::Response
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x43, 0x03, 0x04, 0x05, 0xA2, 0xB4, 0xC6, 0xD8,
                0xF7
            ]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedComponents(
                    crate::NrOfSupportedComponents {
                        buttons: 8,
                        encoders: 2,
                        analog: 2,
                        leds: 8,
                        touchscreen_buttons: 1
                    }
                )),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x4D, 0x08, 0x02, 0x02, 0x08, 0x01, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedPresets(10)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x50, 0x0A, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::BootloaderSupport(true)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x51, 0x01, 0xF7]
        );
    }

    #[test]
    fn should_render_special_messages_with_two_bytes() {
        let renderer = OpenDeckRenderer {
            value_size: ValueSize::TwoBytes,
        };
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::Handshake),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValueSize),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x02, 0x00, 0x02, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::ValuesPerMessage(0x20)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x03, 0x00, 0x20, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersion(FirmwareVersion {
                    major: 0x01,
                    minor: 0x02,
                    revision: 0x03,
                })),
                MessageStatus::Response,
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x56, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::HardwareUID(HardwareUid(0x12345678))),
                MessageStatus::Response
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x42, 0x00, 0x12, 0x00, 0x34, 0x00, 0x56, 0x00,
                0x78, 0xF7
            ]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    FirmwareVersion {
                        major: 0x03,
                        minor: 0x04,
                        revision: 0x05,
                    },
                    HardwareUid(0x06070809)
                )),
                MessageStatus::Response
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x43, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00,
                0x06, 0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0xF7
            ]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedComponents(
                    crate::NrOfSupportedComponents {
                        buttons: 8,
                        encoders: 2,
                        analog: 2,
                        leds: 8,
                        touchscreen_buttons: 1
                    }
                )),
                MessageStatus::Response
            ),
            &[
                0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x4D, 0x00, 0x08, 0x00, 0x02, 0x00, 0x02, 0x00,
                0x08, 0x00, 0x01, 0xF7
            ]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::NrOfSupportedPresets(10)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x50, 0x00, 0x0A, 0xF7]
        );
        assert_eq!(
            renderer.render(
                OpenDeckResponse::Special(SpecialResponse::BootloaderSupport(true)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x51, 0x00, 0x01, 0xF7]
        );
    }

    #[test]
    fn should_render_u16() {
        let buf = Vec::new();
        assert_eq!(ValueSize::TwoBytes.push(10000, buf), &[0x4E, 0x10]);
    }
}
