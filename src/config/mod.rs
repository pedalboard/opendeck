use crate::{
    analog::Analog,
    button::{handler::Action, Button},
    config::backup::ConfigBackupIterator,
    encoder::{handler::EncoderPulse, Encoder},
    global::{GlobalMidi, GlobalPreset, GlobalSection},
    handler::Messages,
    led::Led,
    parser::{OpenDeckParseError, OpenDeckParser},
    renderer::OpenDeckRenderer,
    Amount, Block, HardwareUid, MessageStatus, NewValues, NrOfSupportedComponents, OpenDeckRequest,
    OpenDeckResponse, SpecialRequest, SpecialResponse, ValueSize, Wish,
};

use heapless::Vec;
use midi2::{error::BufferOverflow, sysex7::Sysex7};

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
    for Preset<B, E, A, L>
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
}

pub struct Config<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize> {
    parser: OpenDeckParser,
    global: GlobalConfig,
    enabled: bool,
    presets: Vec<Preset<B, A, E, L>, P>,
    version: FirmwareVersion,
    uid: u32,
    reboot: fn(),
    bootloader: fn(),
}

pub enum SysexResponseIterator<
    'a,
    const P: usize,
    const B: usize,
    const A: usize,
    const E: usize,
    const L: usize,
> {
    Config(ConfigResponseIterator<'a, P, B, A, E, L>),
    Backup(ConfigBackupIterator<'a, P, B, A, E, L>),
    Error(SingleResponseIterator),
    None,
}

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    SysexResponseIterator<'_, P, B, A, E, L>
{
    pub fn next<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<Option<Sysex7<&'buf mut [u8]>>, BufferOverflow> {
        let renderer = OpenDeckRenderer::new(ValueSize::TwoBytes, buffer);
        match self {
            SysexResponseIterator::Config(i) => {
                if let Some(res) = i.next() {
                    #[cfg(feature = "defmt")]
                    defmt::info!("opendeck-res: {}", res);
                    return renderer.render(res, MessageStatus::Response);
                }
                Ok(None)
            }
            SysexResponseIterator::Backup(i) => {
                if let Some(res) = i.next() {
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
    'a,
    const P: usize,
    const B: usize,
    const A: usize,
    const E: usize,
    const L: usize,
> {
    config: &'a mut Config<P, B, A, E, L>,
    index: usize,
    request: OpenDeckRequest,
}

impl<'a, const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    ConfigResponseIterator<'a, P, B, A, E, L>
{
    pub fn new(config: &'a mut Config<P, B, A, E, L>, request: OpenDeckRequest) -> Self {
        ConfigResponseIterator {
            config,
            index: 0,
            request,
        }
    }
}
impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize> Iterator
    for ConfigResponseIterator<'_, P, B, A, E, L>
{
    type Item = OpenDeckResponse;
    fn next(&mut self) -> Option<OpenDeckResponse> {
        if self.index == 0 {
            if let Some(odr) = self.config.process_req(self.request) {
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

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    Config<P, B, A, E, L>
{
    pub fn new(version: FirmwareVersion, uid: u32, reboot: fn(), bootloader: fn()) -> Self {
        let mut presets = Vec::new();
        for _ in 0..P {
            presets.push(Preset::default()).unwrap();
        }

        Config {
            parser: OpenDeckParser::new(ValueSize::TwoBytes),
            enabled: false,
            presets,
            version,
            uid,
            reboot,
            bootloader,
            global: GlobalConfig::default(),
        }
    }
    /// Processes a SysEx request and returns an optional responses.
    pub fn process_sysex(&mut self, request: &[u8]) -> SysexResponseIterator<'_, P, B, A, E, L> {
        let request = self.parser.parse(request);
        match request {
            Ok(OpenDeckRequest::Special(SpecialRequest::Backup)) => {
                SysexResponseIterator::Backup(ConfigBackupIterator::new(self))
            }
            Ok(request) => {
                SysexResponseIterator::Config(ConfigResponseIterator::new(self, request))
            }
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
                (self.bootloader)();
                None
            }
            SpecialRequest::Reboot => {
                (self.reboot)();
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
            SpecialRequest::FactoryReset => None,
            SpecialRequest::Backup => None,
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
                Block::Led(index, section) => match wish {
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

                Block::Display => {}
                Block::Touchscreen => {}
            };
        };

        (res_values, for_amount)
    }

    fn current_preset_mut(&mut self) -> Option<&mut Preset<B, A, E, L>> {
        self.presets.get_mut(self.global.preset.current)
    }

    pub fn handle_button(&mut self, index: usize, action: Action) -> Messages {
        if let Some(preset) = self.current_preset_mut() {
            if let Some(button) = preset.button_mut(index as u16) {
                return Messages::Button(button.handle(action));
            }
        }
        Messages::None
    }
    pub fn handle_analog(&mut self, index: usize, value: u16) -> Messages {
        if let Some(preset) = self.current_preset_mut() {
            if let Some(analog) = preset.analog_mut(index as u16) {
                return Messages::Analog(analog.handle(value));
            }
        }
        Messages::None
    }
    pub fn handle_encoder(&mut self, index: usize, pulse: EncoderPulse) -> Messages {
        if let Some(preset) = self.current_preset_mut() {
            if let Some(encoder) = preset.encoder_mut(index as u16) {
                return Messages::Encoder(encoder.handle(pulse));
            }
        }
        Messages::None
    }
}
#[cfg(test)]
mod tests {
    use crate::MAX_MESSAGE_SIZE;

    use super::*;
    use midi2::Data;

    #[test]
    fn test_process_sysex_handshake() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let reboot = || {};
        let bootloader = || {};
        let mut config = Config::<1, 1, 1, 1, 1>::new(version, uid, reboot, bootloader);

        let request = [0xF0, 0x00, 0x53, 0x43, 0x00, 0x00, 0x01, 0xF7]; // Example SysEx request for handshake
        let mut responses = config.process_sysex(&request);

        let exp = &[0xF0, 0x00, 0x53, 0x43, 0x01, 0x00, 0x01, 0xF7][..];
        let buf = &mut [0; MAX_MESSAGE_SIZE];
        assert_eq!(responses.next(buf).unwrap().unwrap().data(), exp);
    }

    #[test]
    fn test_process_sysex_configuration() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let reboot = || {};
        let bootloader = || {};
        let mut config = Config::<1, 1, 1, 1, 1>::new(version, uid, reboot, bootloader);

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
        assert_eq!(responses.next(buf).unwrap().unwrap().data(), exp);
    }
    #[test]
    fn test_get_all_with_ack() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let reboot = || {};
        let bootloader = || {};
        let mut config = Config::<1, 20, 1, 1, 1>::new(version, uid, reboot, bootloader);

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
        assert_eq!(responses.next(buf).unwrap().unwrap().data(), resp1);
        let resp2 = &[
            0xF0, 0x00, 0x53, 0x43, 0x01, 0x7E, 0x00, 0x01, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00,
            0xF7,
        ];
        assert_eq!(responses.next(buf).unwrap().unwrap().data(), resp2);
        assert_eq!(responses.next(&mut []).unwrap(), None);
    }
}
