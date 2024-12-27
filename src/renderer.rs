pub struct OpenDeckRenderer {}

use crate::{
    ByteOrder, FirmwareVersion, HardwareUid, MessageStatus, OpenDeckResponse, SpecialRequest,
    SpecialResponse, MAX_MESSAGE_SIZE, M_ID_0, M_ID_1, M_ID_2, SYSEX_END, SYSEX_START,
};
use heapless::Vec;

pub type Buffer = Vec<u8, MAX_MESSAGE_SIZE>;

impl OpenDeckRenderer {
    pub fn render(res: OpenDeckResponse, status: MessageStatus) -> Buffer {
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
                SpecialResponse::ValueSize(size) => {
                    buf.push(size as u8).unwrap();
                    SpecialRequest::ValueSize as u8
                }
                SpecialResponse::ValuesPerMessage(vpm) => {
                    buf.push(vpm).unwrap();
                    SpecialRequest::ValuesPerMessage as u8
                }
                SpecialResponse::NrOfSupportedComponents(nr_of_comps) => {
                    buf.push(nr_of_comps as u8).unwrap();
                    SpecialRequest::NrOfSupportedComponents as u8
                }
                SpecialResponse::NrOfSupportedPresets(_) => {
                    SpecialRequest::NrOfSupportedPresets as u8
                }
                SpecialResponse::FirmwareVersion(v) => {
                    buf = v.push(buf);
                    SpecialRequest::FirmwareVersion as u8
                }
                SpecialResponse::HardwareUUID(uid) => {
                    buf = uid.push(buf);
                    SpecialRequest::HardwareUUID as u8
                }
                SpecialResponse::FirmwareVersionAndHardwareUUID(v, uid) => {
                    buf = v.push(buf);
                    buf = uid.push(buf);
                    SpecialRequest::FirmwareVersionAndHardwareUUID as u8
                }
                SpecialResponse::Backup => SpecialRequest::Backup as u8,
                SpecialResponse::BootloaderSupport(_) => SpecialRequest::BootloaderSupport as u8,
            },
        };

        buf.insert(ByteOrder::Wish as usize, wish).unwrap();
        buf.push(SYSEX_END).unwrap();
        buf
    }
}

impl FirmwareVersion {
    fn push(self, mut buf: Buffer) -> Buffer {
        buf.push(self.major).unwrap();
        buf.push(self.minor).unwrap();
        buf.push(self.revision).unwrap();
        buf
    }
}

impl HardwareUid {
    fn push(self, mut buf: Buffer) -> Buffer {
        buf.push(((self.0 >> 24) & 0xff) as u8).unwrap();
        buf.push(((self.0 >> 16) & 0xff) as u8).unwrap();
        buf.push(((self.0 >> 8) & 0xff) as u8).unwrap();
        buf.push(((self.0) & 0xff) as u8).unwrap();
        buf
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{FirmwareVersion, HardwareUid, ValueSize};

    #[test]
    fn should_render_special_messages() {
        assert_eq!(
            OpenDeckRenderer::render(
                OpenDeckResponse::Special(SpecialResponse::Handshake),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7]
        );
        assert_eq!(
            OpenDeckRenderer::render(
                OpenDeckResponse::Special(SpecialResponse::ValueSize(ValueSize::OneByte)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x02, 0x01, 0xF7]
        );
        assert_eq!(
            OpenDeckRenderer::render(
                OpenDeckResponse::Special(SpecialResponse::ValuesPerMessage(0x20)),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x03, 0x20, 0xF7]
        );
        assert_eq!(
            OpenDeckRenderer::render(
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
            OpenDeckRenderer::render(
                OpenDeckResponse::Special(SpecialResponse::HardwareUUID(HardwareUid(0x12345678))),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x42, 0x12, 0x34, 0x56, 0x78, 0xF7]
        );
        assert_eq!(
            OpenDeckRenderer::render(
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
    }
}
