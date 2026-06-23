//! End-to-end tests using the exact byte sequences from the OpenDeck wiki:
//! https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#configuration-examples
//!
//! All tests use the two-byte protocol variant (current firmware standard).

#[cfg(test)]
mod tests {
    use crate::{
        analog::AnalogSection, button::ButtonSection, encoder::EncoderSection, led::LedSection,
        parser::OpenDeckParser, renderer::OpenDeckRenderer, Amount, Block, MessageStatus,
        OpenDeckRequest, OpenDeckResponse, ValueSize, Wish, MAX_MESSAGE_SIZE,
    };
    use heapless::Vec;
    use midi2::prelude::*;

    fn parser() -> OpenDeckParser {
        OpenDeckParser::new(ValueSize::TwoBytes)
    }

    fn render(res: OpenDeckResponse, status: MessageStatus) -> Vec<u8, MAX_MESSAGE_SIZE> {
        let mut buffer = [0u8; MAX_MESSAGE_SIZE];
        let renderer = OpenDeckRenderer::new(ValueSize::TwoBytes, &mut buffer);
        let sysex = renderer.render(res, status).unwrap().unwrap();
        Vec::from_slice(sysex.data()).unwrap()
    }

    // =========================================================================
    // Wiki > Configuration examples > Two byte protocol variant > GET command
    // https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#get-command
    // =========================================================================

    /// Wiki Example #1: GET MIDI ID for analog component 5
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-1-midi-id-for-analog-component-5
    ///
    /// Request: F0 00 53 43 00 00 00 00 03 03 00 05 00 00 F7
    /// Response: F0 00 53 43 01 00 00 00 03 03 00 05 00 00 00 05 F7
    #[test]
    fn test_parse_wiki_get_analog_midi_id() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Configuration(
                Wish::Get,
                Amount::Single,
                Block::Analog(5, AnalogSection::MidiId(0)),
            )
        );
    }

    /// Wiki Example #1 response rendering
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-1-midi-id-for-analog-component-5
    #[test]
    fn test_render_wiki_get_analog_midi_id_response() {
        let expected = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00,
            0x00, 0x05, 0xF7,
        ];
        let rendered = render(
            OpenDeckResponse::Configuration(
                Wish::Get,
                Amount::Single,
                Block::Analog(5, AnalogSection::MidiId(0)),
                Vec::from_slice(&[5]).unwrap(),
            ),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    /// Wiki Example #2: GET message type for all encoders
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-2-message-type-for-all-encoders
    ///
    /// Request: F0 00 53 43 00 00 00 01 02 02 00 00 00 00 F7
    #[test]
    fn test_parse_wiki_get_all_encoder_message_type() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x01, 0x02, 0x02, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Configuration(
                Wish::Get,
                Amount::All(0x00),
                Block::Encoder(
                    0,
                    EncoderSection::MessageType(
                        crate::encoder::EncoderMessageType::ControlChange7Fh01h
                    )
                ),
            )
        );
    }

    /// Wiki Example #3: GET MIDI IDs for all switches (multi-part, MESSAGE_PART=7F)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-3-midi-ids-for-all-switches
    ///
    /// Request: F0 00 53 43 00 7F 00 01 01 02 00 00 00 00 F7
    #[test]
    fn test_parse_wiki_get_all_switch_midi_id_multipart() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x7F, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Configuration(
                Wish::Get,
                Amount::All(0x7F),
                Block::Button(0, ButtonSection::MidiId(0)),
            )
        );
    }

    // =========================================================================
    // Wiki > Configuration examples > Two byte protocol variant > SET command
    // https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#set-command
    // =========================================================================

    /// Wiki SET Example #1: Turn output 1 on (set control type to Static)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-1-turn-output-1-on
    ///
    /// Request: F0 00 53 43 00 00 01 00 04 05 00 00 00 0A F7
    /// Response: F0 00 53 43 01 00 01 00 04 05 00 00 00 0A F7
    #[test]
    fn test_parse_wiki_set_output_control_type_static() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x04, 0x05, 0x00, 0x00, 0x00, 0x0A,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Led(0, LedSection::ControlType(crate::led::ControlType::Static)),
            )
        );
    }

    /// Wiki SET Example #1 response rendering
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-1-turn-output-1-on
    #[test]
    fn test_render_wiki_set_output_control_type_static_response() {
        let expected = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0x00, 0x04, 0x05, 0x00, 0x00, 0x00, 0x0A,
            0xF7,
        ];
        let rendered = render(
            OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Led(0, LedSection::ControlType(crate::led::ControlType::Static)),
                Vec::new(),
            ),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    /// Wiki SET Example #2: Send program change on switch 4
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-2-send-program-change-on-switch-4
    ///
    /// Request: F0 00 53 43 00 00 01 00 01 01 00 04 00 01 F7
    /// Response: F0 00 53 43 01 00 01 00 01 01 00 04 00 01 F7
    #[test]
    fn test_parse_wiki_set_switch_message_type_program_change() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x01, 0x01, 0x00, 0x04, 0x00, 0x01,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(
                    4,
                    ButtonSection::MessageType(crate::button::ButtonMessageType::ProgramChange)
                ),
            )
        );
    }

    /// Wiki SET Example #2 response rendering
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-2-send-program-change-on-switch-4
    #[test]
    fn test_render_wiki_set_switch_message_type_response() {
        let expected = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0x00, 0x01, 0x01, 0x00, 0x04, 0x00, 0x01,
            0xF7,
        ];
        let rendered = render(
            OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(
                    4,
                    ButtonSection::MessageType(crate::button::ButtonMessageType::ProgramChange),
                ),
                Vec::new(),
            ),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    /// Wiki SET Example #3: Set upper CC limit to 4100 for potentiometer 5
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-3-set-upper-cc-limit-to-4100-for-potentiometer-5
    ///
    /// Request: F0 00 53 43 00 00 01 00 03 07 00 05 20 04 F7
    /// Response: F0 00 53 43 01 00 01 00 03 07 00 05 20 04 F7
    ///
    /// Value encoding: 4100 = 0x1004
    ///   high byte = 0x10, low byte = 0x04
    ///   encoded high = (0x10 << 1) | (0x04 >> 7) = 0x20
    ///   encoded low  = 0x04 & 0x7F = 0x04
    #[test]
    fn test_parse_wiki_set_analog_upper_cc_limit() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x03, 0x07, 0x00, 0x05, 0x20, 0x04,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Analog(5, AnalogSection::UpperCCLimit(4100)),
            )
        );
    }

    /// Wiki SET Example #3 response rendering
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-3-set-upper-cc-limit-to-4100-for-potentiometer-5
    #[test]
    fn test_render_wiki_set_analog_upper_cc_limit_response() {
        let expected = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0x00, 0x03, 0x07, 0x00, 0x05, 0x20, 0x04,
            0xF7,
        ];
        let rendered = render(
            OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Analog(5, AnalogSection::UpperCCLimit(4100)),
                Vec::new(),
            ),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    // =========================================================================
    // Wiki > Special requests — two-byte protocol
    // https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#special-requests
    // =========================================================================

    /// Wiki: Handshake request/response
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#handshake-request
    ///
    /// Request: F0 00 53 43 00 00 01 F7
    /// Response: F0 00 53 43 01 00 01 F7
    #[test]
    fn test_parse_wiki_handshake() {
        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Special(crate::SpecialRequest::Handshake)
        );
    }

    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#handshake-request
    #[test]
    fn test_render_wiki_handshake_response() {
        let expected = [0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7];
        let rendered = render(
            OpenDeckResponse::Special(crate::SpecialResponse::Handshake),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    /// Wiki: Restore start
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#restore-start
    ///
    /// Request: F0 00 53 43 00 00 1C F7
    /// Response: F0 00 53 43 01 00 1C F7
    #[test]
    fn test_parse_wiki_restore_start() {
        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x1C, 0xF7];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Special(crate::SpecialRequest::RestoreStart)
        );
    }

    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#restore-start
    #[test]
    fn test_render_wiki_restore_start_response() {
        let expected = [0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x1C, 0xF7];
        let rendered = render(
            OpenDeckResponse::Special(crate::SpecialResponse::RestoreStart),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    /// Wiki: Restore end
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#restore-end
    ///
    /// Request: F0 00 53 43 00 00 1D F7
    /// Response: F0 00 53 43 01 00 1D F7
    #[test]
    fn test_parse_wiki_restore_end() {
        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x1D, 0xF7];
        let result = parser().parse(&request).unwrap();
        assert_eq!(
            result,
            OpenDeckRequest::Special(crate::SpecialRequest::RestoreEnd)
        );
    }

    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#restore-end
    #[test]
    fn test_render_wiki_restore_end_response() {
        let expected = [0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x1D, 0xF7];
        let rendered = render(
            OpenDeckResponse::Special(crate::SpecialResponse::RestoreEnd),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected);
    }

    // =========================================================================
    // Wiki > Component info messages
    // https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#component-info-messages
    //
    // F0 00 53 43 01 00 49 03 00 00 F7
    // =========================================================================

    /// Wiki: Component info message for analog input 1 (index 0)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#component-info-messages
    ///
    /// Message: F0 00 53 43 01 00 49 03 00 00 F7
    /// Sent FROM the board — test rendering only.
    #[test]
    fn test_render_wiki_component_info_analog() {
        let expected = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x49, 0x03, 0x00, 0x00, 0xF7,
        ];
        let mut buffer = [0u8; MAX_MESSAGE_SIZE];
        let renderer = OpenDeckRenderer::new(ValueSize::TwoBytes, &mut buffer);
        let result = renderer.render_component_info(crate::BlockId::Analog, 0);
        match result {
            Ok(Some(sysex)) => assert_eq!(sysex.data(), &expected),
            Ok(None) => panic!("Expected Some, got None"),
            Err(e) => panic!("Render error: {:?}", e),
        }
    }

    // =========================================================================
    // Roundtrip tests: parse request, then render the response, compare bytes
    // https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#two-byte-protocol-variant
    // =========================================================================

    /// Roundtrip: GET analog MIDI ID 5, response with value 5
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-1-midi-id-for-analog-component-5
    #[test]
    fn test_roundtrip_wiki_get_analog_midi_id() {
        let request_bytes = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00,
            0xF7,
        ];
        let expected_response = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00,
            0x00, 0x05, 0xF7,
        ];

        let parsed = parser().parse(&request_bytes).unwrap();
        let OpenDeckRequest::Configuration(wish, amount, block) = parsed else {
            panic!("Expected Configuration");
        };

        let rendered = render(
            OpenDeckResponse::Configuration(wish, amount, block, Vec::from_slice(&[5]).unwrap()),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected_response);
    }

    /// Roundtrip: SET upper CC limit 4100 for pot 5
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-3-set-upper-cc-limit-to-4100-for-potentiometer-5
    #[test]
    fn test_roundtrip_wiki_set_analog_upper_cc_limit() {
        let request_bytes = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x03, 0x07, 0x00, 0x05, 0x20, 0x04,
            0xF7,
        ];
        let expected_response = [
            0xF0u8, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0x00, 0x03, 0x07, 0x00, 0x05, 0x20, 0x04,
            0xF7,
        ];

        let parsed = parser().parse(&request_bytes).unwrap();
        let OpenDeckRequest::Configuration(wish, amount, block) = parsed else {
            panic!("Expected Configuration");
        };

        // SET response echoes the request with ACK status, no extra new_values appended
        let rendered = render(
            OpenDeckResponse::Configuration(wish, amount, block, Vec::new()),
            MessageStatus::Response,
        );
        assert_eq!(rendered.as_slice(), &expected_response);
    }

    // =========================================================================
    // Wiki > Value encoding: split14bit / mergeTo14bit
    // https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#parameter-configuration-bytes
    // =========================================================================

    /// Verify the two-byte encoding for value 4100 (0x1004)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#example-3-set-upper-cc-limit-to-4100-for-potentiometer-5
    #[test]
    fn test_value_encoding_4100() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x03, 0x07, 0x00, 0x05, 0x20, 0x04,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        if let OpenDeckRequest::Configuration(
            _,
            _,
            Block::Analog(5, AnalogSection::UpperCCLimit(v)),
        ) = result
        {
            assert_eq!(v, 4100);
        } else {
            panic!("Unexpected parse result: {:?}", result);
        }
    }

    /// Verify value encoding for 0 (boundary)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#parameter-configuration-bytes
    #[test]
    fn test_value_encoding_zero() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        if let OpenDeckRequest::Configuration(_, _, Block::Analog(0, AnalogSection::MidiId(v))) =
            result
        {
            assert_eq!(v, 0);
        } else {
            panic!("Unexpected parse result: {:?}", result);
        }
    }

    /// Verify value encoding for 0x3FFF (max 14-bit value)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#parameter-configuration-bytes
    ///
    /// 0x3FFF = high 0x3F, low 0xFF
    /// encoded high = (0x3F << 1) | (0xFF >> 7) = 0x7F
    /// encoded low = 0xFF & 0x7F = 0x7F
    #[test]
    fn test_value_encoding_max_14bit() {
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x03, 0x07, 0x00, 0x00, 0x7F, 0x7F,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        if let OpenDeckRequest::Configuration(
            _,
            _,
            Block::Analog(0, AnalogSection::UpperCCLimit(v)),
        ) = result
        {
            assert_eq!(v, 0x3FFF);
        } else {
            panic!("Unexpected parse result: {:?}", result);
        }
    }

    /// Output configuration block > Output state (section 0)
    /// Wiki: Section 0, NEW_VALUE range 0-1 (Off/On)
    /// https://github.com/shanteacontrols/OpenDeck/wiki/Sysex-Configuration#output-state
    #[test]
    fn test_parse_output_state_section() {
        // SET output 0, section 0 (state), value 1 (on)
        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01,
            0xF7,
        ];
        let result = parser().parse(&request).unwrap();
        if let OpenDeckRequest::Configuration(Wish::Set, _, Block::Led(0, LedSection::State(v))) =
            result
        {
            assert_eq!(v, true);
        } else {
            panic!("Expected LedSection::State, got: {:?}", result);
        }
    }

    /// Output section 0 get() must return 0 or 1 (not a Color enum value)
    #[test]
    fn test_led_get_state_returns_bool() {
        use crate::led::Led;
        let mut led = Led::new(0);
        assert_eq!(led.get(LedSection::State(false)), 0);
        led.set(LedSection::State(true));
        assert_eq!(led.get(LedSection::State(false)), 1);
    }
}
