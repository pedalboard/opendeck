pub struct OpenDeckRenderer {}

use crate::{
    ByteOrder, MessageStatus, OpenDeckResponse, SpecialRequest, SpecialResponse, MAX_MESSAGE_SIZE,
    M_ID_0, M_ID_1, M_ID_2, SYSEX_END, SYSEX_START,
};
use heapless::Vec;

impl OpenDeckRenderer {
    pub fn render(res: OpenDeckResponse, status: MessageStatus) -> Vec<u8, MAX_MESSAGE_SIZE> {
        let wish = match res {
            OpenDeckResponse::Special(special) => match special {
                SpecialResponse::Handshake => SpecialRequest::Handshake as u8,
                SpecialResponse::ValueSize(_) => SpecialRequest::ValueSize as u8,
                SpecialResponse::ValuesPerMessage(_) => SpecialRequest::ValuesPerMessage as u8,
                SpecialResponse::NrOfSupportedComponents(_) => {
                    SpecialRequest::NrOfSupportedComponents as u8
                }
                SpecialResponse::NrOfSupportedPresets(_) => {
                    SpecialRequest::NrOfSupportedPresets as u8
                }
                SpecialResponse::Backup => SpecialRequest::Backup as u8,
                SpecialResponse::HardwareUUID(_) => SpecialRequest::HardwareUUID as u8,
                SpecialResponse::BootloaderSupport(_) => SpecialRequest::BootloaderSupport as u8,
                SpecialResponse::FirmwareVersion(_, _, _) => SpecialRequest::FirmwareVersion as u8,
                SpecialResponse::FirmwareVersionAndHardwareUUID(_, _, _, _) => {
                    SpecialRequest::FirmwareVersionAndHardwareUUID as u8
                }
            },
        };
        let mut res = Vec::new();
        res.insert(ByteOrder::Start as usize, SYSEX_START).unwrap();
        res.insert(ByteOrder::Id1 as usize, M_ID_0).unwrap();
        res.insert(ByteOrder::Id2 as usize, M_ID_1).unwrap();
        res.insert(ByteOrder::Id3 as usize, M_ID_2).unwrap();
        res.insert(ByteOrder::Status as usize, status as u8)
            .unwrap();
        res.insert(ByteOrder::Part as usize, 0).unwrap();
        res.insert(ByteOrder::Wish as usize, wish).unwrap();
        res.push(SYSEX_END).unwrap();
        res
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_render_special_messages() {
        assert_eq!(
            OpenDeckRenderer::render(
                OpenDeckResponse::Special(SpecialResponse::Handshake),
                MessageStatus::Response
            ),
            &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7]
        );
    }
}
