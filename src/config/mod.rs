use crate::{
    analog::Analog,
    button::Button,
    encoder::Encoder,
    global::{GlobalMidi, GlobalPreset, GlobalSection},
    led::{GlobalLed, Led},
    parser::{OpenDeckParseError, OpenDeckParser},
    renderer::{Buffer, OpenDeckRenderer},
    Amount, Block, HardwareUid, MessageStatus, NewValues, NrOfSupportedComponents, OpenDeckRequest,
    OpenDeckResponse, SpecialRequest, SpecialResponse, ValueSize, Wish,
};
use midi_types::{Value14, Value7};

// FIXME calculate value based on generic const
const OPENDECK_MAX_NR_MESSAGES: usize = 2;

use heapless::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
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
            buttons.push(Button::new(Value7::new(i as u8))).unwrap();
        }
        let mut encoders = Vec::new();
        for i in 0..E {
            encoders
                .push(Encoder::new(Value14::new(i16::MIN + i as i16)))
                .unwrap();
        }
        let mut analogs = Vec::new();
        for i in 0..A {
            analogs
                .push(Analog::new(Value14::new(i16::MIN + i as i16)))
                .unwrap();
        }
        let mut leds = Vec::new();
        for i in 0..L {
            leds.push(Led::new(Value7::new(i as u8))).unwrap();
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
    fn button_mut(&mut self, index: &u16) -> Option<&mut Button> {
        self.buttons.get_mut(*index as usize)
    }
    fn button(&mut self, index: &u16) -> Option<&Button> {
        self.buttons.get(*index as usize)
    }
    fn encoder_mut(&mut self, index: &u16) -> Option<&mut Encoder> {
        self.encoders.get_mut(*index as usize)
    }
    fn encoder(&mut self, index: &u16) -> Option<&Encoder> {
        self.encoders.get(*index as usize)
    }
    fn analog_mut(&mut self, index: &u16) -> Option<&mut Analog> {
        self.analogs.get_mut(*index as usize)
    }
    fn analog(&mut self, index: &u16) -> Option<&Analog> {
        self.analogs.get(*index as usize)
    }
    fn led_mut(&mut self, index: &u16) -> Option<&mut Led> {
        self.leds.get_mut(*index as usize)
    }
    fn led(&mut self, index: &u16) -> Option<&Led> {
        self.leds.get(*index as usize)
    }
}

#[derive(Default)]
pub struct GlobalConfig {
    led: GlobalLed,
    midi: GlobalMidi,
    preset: GlobalPreset,
}

pub struct Config<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize> {
    global: GlobalConfig,
    enabled: bool,
    presets: Vec<Preset<B, A, E, L>, P>,
    version: FirmwareVersion,
    uid: u32,
    reboot: fn(),
    bootloader: fn(),
}

type Responses = Vec<Buffer, OPENDECK_MAX_NR_MESSAGES>;

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    Config<P, B, A, E, L>
{
    pub fn new(version: FirmwareVersion, uid: u32, reboot: fn(), bootloader: fn()) -> Self {
        let mut presets = Vec::new();
        for _ in 0..P {
            presets.push(Preset::default()).unwrap();
        }

        Config {
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
    pub fn process_sysex(&mut self, request: &[u8]) -> Responses {
        let parser = OpenDeckParser::new(ValueSize::TwoBytes);
        let renderer = OpenDeckRenderer::new(ValueSize::TwoBytes);
        let mut responses = Vec::new();
        match parser.parse(request) {
            Ok(req) => {
                if let Some(odr) = self.process_req(&req) {
                    #[cfg(feature = "defmt")]
                    defmt::info!("opendeck-res: {}", odr);

                    responses
                        .push(renderer.render(odr, MessageStatus::Response))
                        .unwrap();

                    if let OpenDeckRequest::Configuration(wish, Amount::All(0x7E), block) = req {
                        let end = OpenDeckResponse::Configuration(
                            wish,
                            Amount::All(0x7E),
                            block,
                            Vec::new(),
                        );
                        responses
                            .push(renderer.render(end, MessageStatus::Response))
                            .unwrap();
                    }
                }
            }
            Err(OpenDeckParseError::StatusError(status)) => {
                responses
                    .push(renderer.render(
                        OpenDeckResponse::Special(SpecialResponse::Handshake),
                        status,
                    ))
                    .unwrap();
            }
            Err(err) => {
                #[cfg(feature = "defmt")]
                defmt::error!("error parsing sysex message: {}", err)
            }
        }
        responses
    }

    fn process_req(&mut self, req: &OpenDeckRequest) -> Option<OpenDeckResponse> {
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
                    wish.clone(),
                    for_amount,
                    block.clone(),
                    res_values,
                ))
            }
            _ => None,
        }
    }

    fn process_special_req(&mut self, special: &SpecialRequest) -> Option<SpecialResponse> {
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
            SpecialRequest::FirmwareVersion => {
                Some(SpecialResponse::FirmwareVersion(self.version.clone()))
            }
            SpecialRequest::HardwareUID => {
                Some(SpecialResponse::HardwareUID(HardwareUid(self.uid)))
            }
            SpecialRequest::FirmwareVersionAndHardwareUUID => {
                Some(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    self.version.clone(),
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
            _ => None,
        }
    }

    fn process_config(
        &mut self,
        wish: &Wish,
        amount: &Amount,
        block: &Block,
    ) -> (NewValues, Amount) {
        let mut res_values = Vec::new();
        let mut for_amount = amount.clone();

        if let Some(preset) = self.current_preset_mut() {
            match block {
                Block::Global(GlobalSection::Midi(_, _)) => {}
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
}
