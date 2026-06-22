use crate::{
    analog::Analog,
    button::{handler::Action, Button},
    config::backup::ConfigBackupIterator,
    encoder::{handler::EncoderPulse, Encoder},
    global::{GlobalMidi, GlobalPreset, GlobalSection},
    handler::Messages,
    led::{ControlType, Led, LedSection},
    parser::{OpenDeckParseError, OpenDeckParser},
    renderer::{OpenDeckRenderer, RenderError},
    Amount, Block, HardwareUid, MessageStatus, NewValues, NrOfSupportedComponents, OpenDeckRequest,
    OpenDeckResponse, SpecialRequest, SpecialResponse, ValueSize, Wish,
};

use heapless::Vec;
use midi2::sysex7::Sysex7;

mod backup;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub revision: u8,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Preset<const B: usize, const A: usize, const E: usize, const L: usize> {
    buttons: Vec<Button, B>,
    encoders: Vec<Encoder, E>,
    analogs: Vec<Analog, A>,
    leds: Vec<Led, L>,
}

impl<const B: usize, const A: usize, const E: usize, const L: usize> Default
    for Preset<B, A, E, L>
{
    fn default() -> Self {
        let mut buttons = Vec::new();
        for i in 0..B {
            buttons.push(Button::new(i as u8)).unwrap();
        }
        let mut encoders = Vec::new();
        for i in 0..E {
            encoders.push(Encoder::new(i as u16)).unwrap();
        }
        let mut analogs = Vec::new();
        for i in 0..A {
            analogs.push(Analog::new(i as u16)).unwrap();
        }
        let mut leds = Vec::new();
        for i in 0..L {
            leds.push(Led::new(i as u8)).unwrap();
        }

        Preset {
            buttons,
            encoders,
            analogs,
            leds,
        }
    }
}

impl<const B: usize, const A: usize, const E: usize, const L: usize> Preset<B, E, A, L> {
    fn button_mut(&mut self, index: u16) -> Option<&mut Button> {
        self.buttons.get_mut(index as usize)
    }
    fn button(&mut self, index: u16) -> Option<&Button> {
        self.buttons.get(index as usize)
    }
    fn encoder_mut(&mut self, index: u16) -> Option<&mut Encoder> {
        self.encoders.get_mut(index as usize)
    }
    fn encoder(&mut self, index: u16) -> Option<&Encoder> {
        self.encoders.get(index as usize)
    }
    fn analog_mut(&mut self, index: u16) -> Option<&mut Analog> {
        self.analogs.get_mut(index as usize)
    }
    fn analog(&mut self, index: u16) -> Option<&Analog> {
        self.analogs.get(index as usize)
    }
    fn led_mut(&mut self, index: u16) -> Option<&mut Led> {
        self.leds.get_mut(index as usize)
    }
    fn led(&mut self, index: u16) -> Option<&Led> {
        self.leds.get(index as usize)
    }
}

#[derive(Default)]
pub struct GlobalConfig {
    midi: GlobalMidi,
    preset: GlobalPreset,
    led: crate::led::GlobalLed,
}

pub struct Config<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize, H: crate::SystemHandler> {
    parser: OpenDeckParser,
    global: GlobalConfig,
    enabled: bool,
    presets: Vec<Preset<B, A, E, L>, P>,
    version: FirmwareVersion,
    uid: u32,
    serial_number: Vec<u8, 32>,
    handler: H,
}

pub enum SysexResponseIterator<
    const P: usize,
    const B: usize,
    const A: usize,
    const E: usize,
    const L: usize,
> {
    Config(ConfigResponseIterator<P, B, A, E, L>),
    Backup(ConfigBackupIterator<P, B, A, E, L>),
    Error(SingleResponseIterator),
    None,
}

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    SysexResponseIterator<P, B, A, E, L>
{
    pub fn next<'c, H: crate::SystemHandler>(
        &mut self,
        buffer: &'c mut [u8],
        config: &mut Config<P, B, A, E, L, H>,
    ) -> Result<Option<Sysex7<&'c mut [u8]>>, RenderError> {
        let renderer = OpenDeckRenderer::new(ValueSize::TwoBytes, buffer);
        match self {
            SysexResponseIterator::Config(i) => {
                if let Some(res) = i.next(config) {
                    #[cfg(feature = "defmt")]
                    defmt::info!("opendeck-res: {}", res);
                    return renderer.render(res, MessageStatus::Response);
                }
                Ok(None)
            }
            SysexResponseIterator::Backup(i) => {
                if let Some(res) = i.next(config) {
                    #[cfg(feature = "defmt")]
                    defmt::info!("opendeck-bak: {}", res);
                    return renderer.render(res, MessageStatus::Response);
                }
                Ok(None)
            }
            SysexResponseIterator::Error(i) => {
                if let Some((res, status)) = i.next() {
                    #[cfg(feature = "defmt")]
                    defmt::info!("opendeck-err: {} {}", res, status);
                    return renderer.render(res, status);
                }
                Ok(None)
            }
            SysexResponseIterator::None => Ok(None),
        }
    }
}

pub struct SingleResponseIterator {
    response: OpenDeckResponse,
    message_status: MessageStatus,
    done: bool,
}

impl SingleResponseIterator {
    pub fn new(
        response: OpenDeckResponse,
        message_status: MessageStatus,
    ) -> SingleResponseIterator {
        SingleResponseIterator {
            response,
            message_status,
            done: false,
        }
    }
}

impl Iterator for SingleResponseIterator {
    type Item = (OpenDeckResponse, MessageStatus);
    fn next(&mut self) -> Option<(OpenDeckResponse, MessageStatus)> {
        if self.done {
            return None;
        }
        Some((self.response.clone(), self.message_status))
    }
}

pub struct ConfigResponseIterator<
    const P: usize,
    const B: usize,
    const A: usize,
    const E: usize,
    const L: usize,
> {
    index: usize,
    request: OpenDeckRequest,
}

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    ConfigResponseIterator<P, B, A, E, L>
{
    pub fn new(request: OpenDeckRequest) -> Self {
        ConfigResponseIterator { index: 0, request }
    }
    fn next<H: crate::SystemHandler>(&mut self, config: &mut Config<P, B, A, E, L, H>) -> Option<OpenDeckResponse> {
        if self.index == 0 {
            if let Some(odr) = config.process_req(self.request) {
                self.index += 1;
                return Some(odr);
            }
        }
        if self.index == 1 {
            // FIXME The assumption is that the maximum amount of values is < 32 and therefore we
            // can fit all values in the first message. The 2nd message below is the final ACK
            // response to mark the ALL values reponse as completed..
            if let OpenDeckRequest::Configuration(wish, Amount::All(0x7E), block) = self.request {
                let ack =
                    OpenDeckResponse::Configuration(wish, Amount::All(0x7E), block, Vec::new());

                self.index += 1;
                return Some(ack);
            }
        }
        None
    }
}

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize, H: crate::SystemHandler>
    Config<P, B, A, E, L, H>
{
    pub fn new(version: FirmwareVersion, uid: u32, handler: H) -> Self {
        Self::new_with_adc_max(version, uid, handler, 4095)
    }

    pub fn new_with_adc_max(version: FirmwareVersion, uid: u32, handler: H, adc_max: u16) -> Self {
        let mut presets = Vec::new();
        for _ in 0..P {
            let mut preset = Preset::default();
            for analog in preset.analogs.iter_mut() {
                analog.set_adc_max(adc_max);
            }
            presets.push(preset).unwrap();
        }

        Config {
            parser: OpenDeckParser::new(ValueSize::TwoBytes),
            enabled: false,
            presets,
            version,
            uid,
            serial_number: Vec::new(),
            handler,
            global: GlobalConfig::default(),
        }
    }

    pub fn set_serial_number(&mut self, serial: &[u8]) {
        self.serial_number.clear();
        for &b in serial.iter().take(32) {
            self.serial_number.push(b).ok();
        }
    }
    /// Processes a SysEx request and returns an optional response.
    pub fn process_sysex(&mut self, request: &[u8]) -> SysexResponseIterator<P, B, A, E, L> {
        let request = self.parser.parse(request);
        match request {
            Ok(OpenDeckRequest::Special(SpecialRequest::Backup)) => {
                SysexResponseIterator::Backup(ConfigBackupIterator::new())
            }
            Ok(request) => SysexResponseIterator::Config(ConfigResponseIterator::new(request)),
            Err(OpenDeckParseError::StatusError(message_status)) => {
                let response = OpenDeckResponse::Special(SpecialResponse::Handshake);
                SysexResponseIterator::Error(SingleResponseIterator::new(response, message_status))
            }
            Err(_err) => {
                #[cfg(feature = "defmt")]
                defmt::error!("error parsing sysex message: {}", _err);
                SysexResponseIterator::None
            }
        }
    }

    pub fn process_req(&mut self, req: OpenDeckRequest) -> Option<OpenDeckResponse> {
        #[cfg(feature = "defmt")]
        defmt::info!("opendeck-req: {}", req);
        match req {
            OpenDeckRequest::Special(special) => {
                if let Some(spec_res) = self.process_special_req(special) {
                    return Some(OpenDeckResponse::Special(spec_res));
                }
                None
            }
            OpenDeckRequest::Configuration(wish, amount, block) => {
                let (res_values, for_amount) = self.process_config(wish, amount, block);
                Some(OpenDeckResponse::Configuration(
                    wish, for_amount, block, res_values,
                ))
            }
            // FIXME support OpenDeckRequest::ComponentInfo
            OpenDeckRequest::ComponentInfo => None,
        }
    }

    fn process_special_req(&mut self, special: SpecialRequest) -> Option<SpecialResponse> {
        match special {
            SpecialRequest::BootloaderMode => {
                self.handler.bootloader();
                None
            }
            SpecialRequest::Reboot => {
                self.handler.reboot();
                None
            }
            SpecialRequest::Handshake => {
                self.enabled = true;
                Some(SpecialResponse::Handshake)
            }
            SpecialRequest::ValueSize => Some(SpecialResponse::ValueSize),
            SpecialRequest::ValuesPerMessage => Some(SpecialResponse::ValuesPerMessage(32)),
            SpecialRequest::FirmwareVersion => Some(SpecialResponse::FirmwareVersion(self.version)),
            SpecialRequest::HardwareUID => {
                Some(SpecialResponse::HardwareUID(HardwareUid(self.uid)))
            }
            SpecialRequest::FirmwareVersionAndHardwareUUID => {
                Some(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    self.version,
                    HardwareUid(self.uid),
                ))
            }
            SpecialRequest::BootloaderSupport => Some(SpecialResponse::BootloaderSupport(true)),
            SpecialRequest::NrOfSupportedPresets => Some(SpecialResponse::NrOfSupportedPresets(P)),
            SpecialRequest::NrOfSupportedComponents => Some(
                SpecialResponse::NrOfSupportedComponents(NrOfSupportedComponents {
                    buttons: B,
                    encoders: E,
                    analog: A,
                    leds: L,
                    touchscreen_buttons: 0,
                }),
            ),
            SpecialRequest::FactoryReset => {
                self.handler.factory_reset();
                None
            }
            SpecialRequest::Backup => None,
            SpecialRequest::SerialNumber => {
                Some(SpecialResponse::SerialNumber(self.serial_number.clone()))
            }
            SpecialRequest::RestoreStart => Some(SpecialResponse::RestoreStart),
            SpecialRequest::RestoreEnd => Some(SpecialResponse::RestoreEnd),
        }
    }

    fn process_config(&mut self, wish: Wish, amount: Amount, block: Block) -> (NewValues, Amount) {
        let mut res_values = Vec::new();
        let mut for_amount = amount;

        if let Some(preset) = self.current_preset_mut() {
            match block {
                Block::Global(GlobalSection::Midi(i, value)) => match wish {
                    Wish::Set => self.global.midi.set(i, value),
                    Wish::Get | Wish::Backup => {
                        res_values.push(self.global.midi.get(i)).unwrap();
                    }
                },
                Block::Global(GlobalSection::Presets(pi, value)) => match wish {
                    Wish::Set => self.global.preset.set(pi, value),
                    Wish::Get | Wish::Backup => {
                        res_values.push(self.global.preset.get(pi)).unwrap();
                    }
                },
                Block::Global(GlobalSection::OSC(_, _)) => {}
                Block::Global(GlobalSection::MDNS(_, _)) => {}
                Block::Global(GlobalSection::ConfigurationUnlock(_, _)) => {}
                Block::Button(index, section) => match wish {
                    Wish::Set => {
                        if let Some(b) = preset.button_mut(index) {
                            b.set(section)
                        }
                    }
                    Wish::Get | Wish::Backup => match amount {
                        Amount::Single => {
                            if let Some(b) = preset.button(index) {
                                res_values.push(b.get(section)).unwrap();
                            }
                        }
                        Amount::All(_) => {
                            for b in preset.buttons.iter() {
                                res_values.push(b.get(section)).unwrap();
                            }
                            for_amount = Amount::All(0)
                        }
                    },
                },
                Block::Encoder(index, section) => match wish {
                    Wish::Set => {
                        if let Some(b) = preset.encoder_mut(index) {
                            b.set(section)
                        }
                    }
                    Wish::Get | Wish::Backup => match amount {
                        Amount::Single => {
                            if let Some(b) = preset.encoder(index) {
                                res_values.push(b.get(section)).unwrap();
                            }
                        }
                        Amount::All(_) => {
                            for b in preset.encoders.iter() {
                                res_values.push(b.get(section)).unwrap();
                            }
                            for_amount = Amount::All(0)
                        }
                    },
                },
                Block::Analog(index, section) => match wish {
                    Wish::Set => {
                        if let Some(b) = preset.analog_mut(index) {
                            b.set(section)
                        }
                    }
                    Wish::Get | Wish::Backup => match amount {
                        Amount::Single => {
                            if let Some(b) = preset.analog(index) {
                                res_values.push(b.get(section)).unwrap();
                            }
                        }
                        Amount::All(_) => {
                            for b in preset.analogs.iter() {
                                res_values.push(b.get(section)).unwrap();
                            }
                            for_amount = Amount::All(0)
                        }
                    },
                },
                Block::Led(index, section) => match section {
                    LedSection::Global(value) => {
                        if let Ok(led_index) = crate::led::LedIndex::try_from(index) {
                            match wish {
                                Wish::Set => self.global.led.set(led_index, &value),
                                Wish::Get | Wish::Backup => {
                                    res_values.push(self.global.led.get(&led_index)).unwrap();
                                }
                            }
                        }
                    }
                    _ => match wish {
                        Wish::Set => {
                            if let Some(b) = preset.led_mut(index) {
                                b.set(section)
                            }
                        }
                        Wish::Get | Wish::Backup => match amount {
                            Amount::Single => {
                                if let Some(b) = preset.led(index) {
                                    res_values.push(b.get(section)).unwrap();
                                }
                            }
                            Amount::All(_) => {
                                for b in preset.leds.iter() {
                                    res_values.push(b.get(section)).unwrap();
                                }
                                for_amount = Amount::All(0)
                            }
                        },
                    },
                },

                Block::Display => {}
                Block::Touchscreen => {}
            };
        };

        (res_values, for_amount)
    }

    fn current_preset_mut(&mut self) -> Option<&mut Preset<B, A, E, L>> {
        self.presets.get_mut(self.global.preset.current)
    }

    pub fn handle_button(&mut self, index: usize, action: Action) -> Messages<'_> {
        use crate::button::ButtonMessageType;

        // Check for internal preset change before borrowing for MIDI handling
        if matches!(action, Action::Pressed) {
            if let Some(preset) = self.presets.get(self.global.preset.current) {
                if let Some(button) = preset.buttons.get(index) {
                    let msg_type = ButtonMessageType::try_from(
                        button.get(crate::button::ButtonSection::MessageType(ButtonMessageType::default()))
                    );
                    if matches!(msg_type, Ok(ButtonMessageType::OpenDeckPresetChange)) {
                        let target = button.get(crate::button::ButtonSection::MidiId(0)) as usize;
                        self.global.preset.current = target;
                        return Messages::None;
                    }
                }
            }
        }

        let channel_override = if self.global.midi.use_global_channel() {
            Some(self.global.midi.global_channel())
        } else {
            None
        };
        let standard_note_off = self.global.midi.standard_note_off();
        if let Some(preset) = self.current_preset_mut() {
            if let Some(button) = preset.button_mut(index as u16) {
                return Messages::Button(button.handle_with_options(action, standard_note_off, channel_override));
            }
        }
        Messages::None
    }
    pub fn handle_analog(&mut self, index: usize, value: u16) -> Messages<'_> {
        let channel_override = if self.global.midi.use_global_channel() {
            Some(self.global.midi.global_channel())
        } else {
            None
        };
        if let Some(preset) = self.current_preset_mut() {
            if let Some(analog) = preset.analog_mut(index as u16) {
                return Messages::Analog(analog.handle_with_channel(value, channel_override));
            }
        }
        Messages::None
    }
    pub fn handle_encoder(&mut self, index: usize, pulse: EncoderPulse) -> Messages<'_> {
        use crate::encoder::EncoderMessageType;

        // Check for internal preset change
        if let Some(preset) = self.presets.get(self.global.preset.current) {
            if let Some(encoder) = preset.encoders.get(index) {
                let msg_type = EncoderMessageType::try_from(
                    encoder.get(crate::encoder::EncoderSection::MessageType(EncoderMessageType::default()))
                );
                if matches!(msg_type, Ok(EncoderMessageType::PresetChange)) {
                    match pulse {
                        EncoderPulse::Clockwise => {
                            if self.global.preset.current + 1 < P {
                                self.global.preset.current += 1;
                            }
                        }
                        EncoderPulse::CounterClockwise => {
                            self.global.preset.current = self.global.preset.current.saturating_sub(1);
                        }
                    }
                    return Messages::None;
                }
            }
        }

        let channel_override = if self.global.midi.use_global_channel() {
            Some(self.global.midi.global_channel())
        } else {
            None
        };
        if let Some(preset) = self.current_preset_mut() {
            if let Some(encoder) = preset.encoder_mut(index as u16) {
                return Messages::Encoder(encoder.handle_with_channel(pulse, channel_override));
            }
        }
        Messages::None
    }

    /// Notify the config that a local MIDI message was generated.
    /// This updates output states for outputs configured in Local control mode.
    pub fn notify_local_midi(&mut self, channel: u8, id: u8, value: u8, is_note_on: bool, is_cc: bool) -> usize {
        self.update_outputs(channel, id, value, is_note_on, true, is_cc)
    }

    /// Notify the config that an external MIDI message was received.
    /// This updates output states for outputs configured in MIDI In control mode.
    pub fn notify_external_midi(&mut self, channel: u8, id: u8, value: u8, is_note_on: bool, is_cc: bool) -> usize {
        self.update_outputs(channel, id, value, is_note_on, false, is_cc)
    }

    fn update_outputs(&mut self, channel: u8, id: u8, value: u8, is_note_on: bool, is_local: bool, is_cc: bool) -> usize {
        use crate::led::handler::OutputState;
        use crate::led::ControlType;

        let Some(preset) = self.presets.get_mut(self.global.preset.current) else {
            return 0;
        };

        let mut count = 0;
        for led in preset.leds.iter_mut() {
            let ct = led.get_control_type();
            let check = match (is_local, is_cc, ct) {
                (true, false, ControlType::LocalNoteSingleValue | ControlType::LocalNoteMultiValue) => true,
                (true, true, ControlType::LocalCcSingleValue | ControlType::LocalCcMultiValue) => true,
                (false, false, ControlType::MidiInNoteSingleValue | ControlType::MidiInNoteMultiValue) => true,
                (false, true, ControlType::MidiInCcSingleValue | ControlType::MidiInCcMultiValue) => true,
                (_, _, ControlType::Static) => true,
                _ => false,
            };
            if !check {
                continue;
            }

            match led.process_midi(channel, id, value, is_note_on) {
                OutputState::On => { led.set_state(true); count += 1; }
                OutputState::Off => { led.set_state(false); count += 1; }
                OutputState::Level(l) => { led.set_level(l); led.set_state(l > 0); count += 1; }
                OutputState::NoChange => {}
            }
        }
        count
    }

    /// Get the current on/off state of an output.
    pub fn output_state(&self, index: usize) -> bool {
        self.presets
            .get(self.global.preset.current)
            .and_then(|p| p.leds.get(index))
            .map(|led| led.is_on())
            .unwrap_or(false)
    }

    /// Get the current level of an output (0-127, for multi-value modes).
    pub fn output_level(&self, index: usize) -> u8 {
        self.presets
            .get(self.global.preset.current)
            .and_then(|p| p.leds.get(index))
            .map(|led| led.get_level())
            .unwrap_or(0)
    }

    /// Get the control type of an output.
    pub fn output_control_type(&self, index: usize) -> ControlType {
        self.presets
            .get(self.global.preset.current)
            .and_then(|p| p.leds.get(index))
            .map(|led| led.get_control_type())
            .unwrap_or_default()
    }

    /// Get the configured color of an output.
    pub fn output_color(&self, index: usize) -> crate::led::Color {
        self.presets
            .get(self.global.preset.current)
            .and_then(|p| p.leds.get(index))
            .map(|led| led.get_color())
            .unwrap_or_default()
    }

    /// Number of configured outputs.
    pub fn output_count(&self) -> usize {
        self.presets
            .get(self.global.preset.current)
            .map(|p| p.leds.len())
            .unwrap_or(0)
    }

    /// Access global MIDI settings (routing, standard note off, etc.)
    pub fn global_midi(&self) -> &GlobalMidi {
        &self.global.midi
    }

    /// Access global LED settings (blink with clock, startup animation, etc.)
    pub fn global_led(&self) -> &crate::led::GlobalLed {
        &self.global.led
    }

    /// Whether a SysEx configuration session is active (handshake received).
    pub fn sysex_enabled(&self) -> bool {
        self.enabled
    }

    /// Current active preset index.
    pub fn active_preset(&self) -> usize {
        self.global.preset.current
    }
}
#[cfg(test)]
mod tests {
    use crate::MAX_MESSAGE_SIZE;

    use super::*;
    use midi2::Data;

    struct NoopHandler;
    impl crate::SystemHandler for NoopHandler {
        fn reboot(&self) {}
        fn bootloader(&self) {}
        fn factory_reset(&self) {}
    }

    #[test]
    fn test_process_sysex_handshake() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let mut config = Config::<1, 1, 1, 1, 1, _>::new(version, uid, NoopHandler);

        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7]; // Example SysEx request for handshake
        let mut responses = config.process_sysex(&request);

        let exp = &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7][..];
        let buf = &mut [0; MAX_MESSAGE_SIZE];
        assert_eq!(
            responses.next(buf, &mut config).unwrap().unwrap().data(),
            exp
        );
    }

    #[test]
    fn test_process_sysex_configuration() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let mut config = Config::<1, 1, 1, 1, 1, _>::new(version, uid, NoopHandler);

        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        let mut responses = config.process_sysex(&request);

        let exp = &[
            0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xF7,
        ];
        let buf = &mut [0; MAX_MESSAGE_SIZE];
        assert_eq!(
            responses.next(buf, &mut config).unwrap().unwrap().data(),
            exp
        );
    }
    #[test]
    fn test_get_all_with_ack() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let mut config = Config::<1, 20, 1, 1, 1, _>::new(version, uid, NoopHandler);

        let request = [
            0xF0, 0x00, 0x53, 0x43, 0x00, 0x7E, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        let mut responses = config.process_sysex(&request);
        let buf = &mut [0; MAX_MESSAGE_SIZE];
        let resp1 = [
            0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06,
            0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0A, 0x00, 0x0B, 0x00, 0x0C, 0x00, 0x0D,
            0x00, 0x0E, 0x00, 0x0F, 0x00, 0x10, 0x00, 0x11, 0x00, 0x12, 0x00, 0x13, 0xF7,
        ];
        assert_eq!(
            responses.next(buf, &mut config).unwrap().unwrap().data(),
            resp1
        );
        let resp2 = &[
            0xF0, 0x00, 0x53, 0x43, 0x01, 0x7E, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        assert_eq!(
            responses.next(buf, &mut config).unwrap().unwrap().data(),
            resp2
        );
        assert_eq!(responses.next(&mut [], &mut config).unwrap(), None);
    }

    #[test]
    fn test_local_loopback_output() {
        use crate::led::{ControlType, LedSection};
        use crate::ChannelOrAll;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 2, 2, 2, 2, _> = Config::new(version, 0, NoopHandler);

        // Directly configure output 0 on the preset
        let preset = config.current_preset_mut().unwrap();
        let led = preset.led_mut(0).unwrap();
        led.set(LedSection::ControlType(ControlType::LocalNoteSingleValue));
        led.set(LedSection::ActivationId(0));
        led.set(LedSection::ActivationValue(127));
        led.set(LedSection::Channel(ChannelOrAll::Channel(1)));

        // Output should be off initially
        assert!(!config.output_state(0));
        assert_eq!(config.output_count(), 2);

        // Simulate local Note On (note 0, velocity 127, channel 1)
        let changed = config.notify_local_midi(1, 0, 127, true, false);
        assert_eq!(changed, 1, "expected 1 output to change");
        assert!(config.output_state(0));

        // Simulate local Note Off
        config.notify_local_midi(1, 0, 0, false, false);
        assert!(!config.output_state(0));
    }

    #[test]
    fn test_preset_default_with_different_analog_and_encoder_counts() {
        // Config<P=1, B=6, A=2, E=4, L=8> — A != E exposes the generic param swap
        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let config: Config<1, 6, 2, 4, 8, _> = Config::new(version, 0, NoopHandler);
        let preset = config.presets.get(0).unwrap();
        assert_eq!(preset.buttons.len(), 6);
        assert_eq!(preset.analogs.len(), 2);
        assert_eq!(preset.encoders.len(), 4);
        assert_eq!(preset.leds.len(), 8);
    }

    /// Wiki: Global > MIDI settings > "Use global MIDI channel"
    /// When enabled, specified global MIDI channel will be used for all components.
    /// Individual channel settings for components will be ignored.
    #[test]
    fn test_global_midi_channel_overrides_button_channel() {
        use crate::button::{ButtonMessageType, ButtonSection};
        use crate::global::MidiIndex;
        use crate::ChannelOrAll;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 2, 1, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure button 0 on channel 1 (0-based) with Note message
        let preset = config.current_preset_mut().unwrap();
        let b = preset.button_mut(0).unwrap();
        b.set(ButtonSection::MessageType(ButtonMessageType::Notes));
        b.set(ButtonSection::Channel(ChannelOrAll::Channel(0))); // channel 1 (0-based)
        b.set(ButtonSection::Value(127));

        // Enable global MIDI channel and set it to channel 5 (wire format 1-based)
        config.global.midi.set(MidiIndex::UseGlobalMIDIchannel, 1);
        config.global.midi.set(MidiIndex::GlobalMIDIchannel, 5); // wire 5 → Channel(5) → MIDI ch 5

        // Button press should use global channel (5) not per-component channel (0)
        let mut buf = [0u8; 8];
        let mut messages = config.handle_button(0, Action::Pressed);
        let msg = messages.next(&mut buf).unwrap().unwrap();
        // Note On on channel 5 = status 0x95
        assert_eq!(msg.data()[0] & 0xF0, 0x90); // Note On
        assert_eq!(msg.data()[0] & 0x0F, 5);    // channel 5
    }

    /// Wiki: Global > MIDI settings > "Standard note off"
    /// When enabled, standard MIDI note off will be sent.
    /// If disabled, note off is sent as note on event with velocity 0.
    #[test]
    fn test_standard_note_off_enabled_sends_real_note_off() {
        use crate::button::{ButtonMessageType, ButtonSection, ButtonType};
        use crate::global::MidiIndex;
        use crate::ChannelOrAll;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 2, 1, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure button 0 as momentary Note
        let preset = config.current_preset_mut().unwrap();
        let b = preset.button_mut(0).unwrap();
        b.set(ButtonSection::Type(ButtonType::Momentary));
        b.set(ButtonSection::MessageType(ButtonMessageType::Notes));
        b.set(ButtonSection::Channel(ChannelOrAll::Channel(0)));
        b.set(ButtonSection::MidiId(60));
        b.set(ButtonSection::Value(100));

        // Enable standard note off
        config.global.midi.set(MidiIndex::StandardNoteOff, 1);

        // Press → Note On
        let mut buf = [0u8; 8];
        let mut messages = config.handle_button(0, Action::Pressed);
        let msg = messages.next(&mut buf).unwrap().unwrap();
        assert_eq!(msg.data(), &[0x90, 60, 100]);

        // Release → should send real Note Off (0x80) not Note On vel=0 (0x90 vel=0)
        let mut messages = config.handle_button(0, Action::Released);
        let msg = messages.next(&mut buf).unwrap().unwrap();
        assert_eq!(msg.data()[0], 0x80); // Note Off status
        assert_eq!(msg.data()[1], 60);   // same note
    }

    /// Verify that the global channel override also works for encoders
    #[test]
    fn test_global_midi_channel_overrides_encoder_channel() {
        use crate::encoder::{handler::EncoderPulse, EncoderSection, EncoderMessageType};
        use crate::global::MidiIndex;
        use crate::ChannelOrAll;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 1, 1, 2, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure encoder 0: enabled, CC 7-bit, channel 2 (0-based = 1)
        let preset = config.current_preset_mut().unwrap();
        let e = preset.encoder_mut(0).unwrap();
        e.set(EncoderSection::Enabled(true));
        e.set(EncoderSection::MessageType(EncoderMessageType::ControlChange));
        e.set(EncoderSection::Channel(ChannelOrAll::Channel(1))); // ch 2
        e.set(EncoderSection::PulsesPerStep(1));

        // Enable global channel = 10 (wire format)
        config.global.midi.set(MidiIndex::UseGlobalMIDIchannel, 1);
        config.global.midi.set(MidiIndex::GlobalMIDIchannel, 10); // wire 10 → Channel(10) → MIDI ch 10

        let mut buf = [0u8; 8];
        let mut messages = config.handle_encoder(0, EncoderPulse::Clockwise);
        let msg = messages.next(&mut buf).unwrap().unwrap();
        // CC on channel 10 = status 0xBA
        assert_eq!(msg.data()[0], 0xBA);
    }

    /// Verify that the global channel override also works for analog
    #[test]
    fn test_global_midi_channel_overrides_analog_channel() {
        use crate::analog::{AnalogSection, AnalogMessageType};
        use crate::global::MidiIndex;
        use crate::ChannelOrAll;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 1, 2, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure analog 0: enabled, CC 7-bit, channel 3 (0-based = 2)
        let preset = config.current_preset_mut().unwrap();
        let a = preset.analog_mut(0).unwrap();
        a.set(AnalogSection::Enabled(true));
        a.set(AnalogSection::MessageType(AnalogMessageType::PotentiometerWithCCMessage7Bit));
        a.set(AnalogSection::Channel(ChannelOrAll::Channel(2)));

        // Enable global channel = 7 (wire format)
        config.global.midi.set(MidiIndex::UseGlobalMIDIchannel, 1);
        config.global.midi.set(MidiIndex::GlobalMIDIchannel, 7); // wire 7 → Channel(7) → MIDI ch 7

        let mut buf = [0u8; 8];
        let mut messages = config.handle_analog(0, 2048); // mid-range ADC value
        let msg = messages.next(&mut buf).unwrap().unwrap();
        // CC on channel 7 = status 0xB7
        assert_eq!(msg.data()[0], 0xB7);
    }

    /// Verify that disabling global channel reverts to per-component channel
    #[test]
    fn test_global_channel_disabled_uses_component_channel() {
        use crate::button::{ButtonMessageType, ButtonSection};
        use crate::global::MidiIndex;
        use crate::ChannelOrAll;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 2, 1, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure button 0 on channel 3
        let preset = config.current_preset_mut().unwrap();
        let b = preset.button_mut(0).unwrap();
        b.set(ButtonSection::MessageType(ButtonMessageType::Notes));
        b.set(ButtonSection::Channel(ChannelOrAll::Channel(3)));
        b.set(ButtonSection::Value(127));

        // Enable global channel, use it, then disable it
        config.global.midi.set(MidiIndex::UseGlobalMIDIchannel, 1);
        config.global.midi.set(MidiIndex::GlobalMIDIchannel, 10);

        let mut buf = [0u8; 8];
        let mut messages = config.handle_button(0, Action::Pressed);
        let msg = messages.next(&mut buf).unwrap().unwrap();
        assert_eq!(msg.data()[0] & 0x0F, 10); // global channel

        // Now disable global channel
        config.global.midi.set(MidiIndex::UseGlobalMIDIchannel, 0);

        let mut messages = config.handle_button(0, Action::Pressed);
        let msg = messages.next(&mut buf).unwrap().unwrap();
        assert_eq!(msg.data()[0] & 0x0F, 3); // back to component channel
    }

    /// Wiki: Special requests > Serial number
    /// Request: F0 00 53 43 00 00 53 F7
    /// Response contains serial number bytes encoded as two-byte values
    #[test]
    fn test_serial_number_request() {
        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config = Config::<1, 1, 1, 1, 1, _>::new(version, 0, NoopHandler);
        config.set_serial_number(&[0xAB, 0xCD, 0xEF, 0x12]);

        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x53, 0xF7];
        let mut responses = config.process_sysex(&request);
        let buf = &mut [0; MAX_MESSAGE_SIZE];
        let resp = responses.next(buf, &mut config).unwrap().unwrap();
        let data = resp.data();
        assert_eq!(data[0], 0xF0);
        assert_eq!(data[6], 0x53); // serial number special ID
        // First byte 0xAB encoded as two bytes: high=0x01, low=0x2B
        assert_eq!(data[7], 0x01);
        assert_eq!(data[8], 0x2B);
        assert_eq!(*data.last().unwrap(), 0xF7);
    }

    /// Factory reset (wiki: Special requests > Factory reset)
    /// Request:  F0 00 53 43 00 00 44 F7
    /// Response: F0 00 53 43 01 00 44 F7
    #[test]
    fn test_factory_reset_calls_handler() {
        use core::sync::atomic::{AtomicBool, Ordering};

        static CALLED: AtomicBool = AtomicBool::new(false);

        struct TrackingHandler;
        impl crate::SystemHandler for TrackingHandler {
            fn reboot(&self) {}
            fn bootloader(&self) {}
            fn factory_reset(&self) {
                CALLED.store(true, Ordering::Relaxed);
            }
        }

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config = Config::<1, 1, 1, 1, 1, _>::new(version, 0, TrackingHandler);

        // Handshake first (required to enable SysEx)
        config.process_sysex(&[0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7]);

        CALLED.store(false, Ordering::Relaxed);
        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x44, 0xF7];
        let mut responses = config.process_sysex(&request);
        let buf = &mut [0; MAX_MESSAGE_SIZE];
        let _ = responses.next(buf, &mut config);

        assert!(CALLED.load(Ordering::Relaxed), "factory_reset() should have been called");
    }

    /// Output configuration block > Global settings (wiki)
    /// SET Block::Led(0, LedSection::Global(1)) should store blink_with_midi_clock = true
    #[test]
    fn test_global_led_settings_stored_and_retrievable() {
        use crate::led::LedSection;
        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<1, 1, 1, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // SET global LED setting: BlinkWithMIDIClock (index 0) = 1
        config.process_req(OpenDeckRequest::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Led(0, LedSection::Global(1)),
        ));

        // GET should return 1
        let res = config.process_req(OpenDeckRequest::Configuration(
            Wish::Get,
            Amount::Single,
            Block::Led(0, LedSection::Global(0)),
        ));
        assert_eq!(
            res,
            Some(OpenDeckResponse::Configuration(
                Wish::Get,
                Amount::Single,
                Block::Led(0, LedSection::Global(0)),
                Vec::from_slice(&[1]).unwrap(),
            ))
        );
    }

    /// Button message type OpenDeckPresetChange (0x11) should switch active preset
    #[test]
    fn test_button_preset_change() {
        use crate::button::{ButtonMessageType, ButtonSection};
        use crate::button::handler::Action;

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<3, 2, 1, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure button 0 as preset change to preset 1
        config.process_req(OpenDeckRequest::Configuration(
            Wish::Set, Amount::Single,
            Block::Button(0, ButtonSection::MessageType(ButtonMessageType::OpenDeckPresetChange)),
        ));
        config.process_req(OpenDeckRequest::Configuration(
            Wish::Set, Amount::Single,
            Block::Button(0, ButtonSection::MidiId(1)),
        ));

        assert_eq!(config.active_preset(), 0);

        // Press the button
        config.handle_button(0, Action::Pressed);

        assert_eq!(config.active_preset(), 1);
    }

    /// Encoder message type PresetChange (0x4) should inc/dec active preset
    #[test]
    fn test_encoder_preset_change() {
        use crate::encoder::{EncoderMessageType, EncoderSection, handler::EncoderPulse};

        let version = FirmwareVersion { major: 1, minor: 0, revision: 0 };
        let mut config: Config<3, 1, 1, 1, 1, _> = Config::new(version, 0, NoopHandler);

        // Configure encoder 0 as preset change on all presets
        for preset in 0..3usize {
            config.global.preset.current = preset;
            config.process_req(OpenDeckRequest::Configuration(
                Wish::Set, Amount::Single,
                Block::Encoder(0, EncoderSection::Enabled(true)),
            ));
            config.process_req(OpenDeckRequest::Configuration(
                Wish::Set, Amount::Single,
                Block::Encoder(0, EncoderSection::MessageType(EncoderMessageType::PresetChange)),
            ));
        }
        config.global.preset.current = 0;

        assert_eq!(config.active_preset(), 0);

        // Clockwise → increment
        config.handle_encoder(0, EncoderPulse::Clockwise);
        assert_eq!(config.active_preset(), 1);

        config.handle_encoder(0, EncoderPulse::Clockwise);
        assert_eq!(config.active_preset(), 2);

        // Clamp at max (P-1 = 2)
        config.handle_encoder(0, EncoderPulse::Clockwise);
        assert_eq!(config.active_preset(), 2);

        // Counter-clockwise → decrement
        config.handle_encoder(0, EncoderPulse::CounterClockwise);
        assert_eq!(config.active_preset(), 1);

        // Clamp at 0
        config.handle_encoder(0, EncoderPulse::CounterClockwise);
        config.handle_encoder(0, EncoderPulse::CounterClockwise);
        assert_eq!(config.active_preset(), 0);
    }
}
