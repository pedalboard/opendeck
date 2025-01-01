use crate::{
    analog::Analog,
    button::Button,
    encoder::Encoder,
    global::{GlobalSection, PresetIndex},
    parser::{OpenDeckParseError, OpenDeckParser},
    renderer::{Buffer, OpenDeckRenderer},
    Amount, Block, FirmwareVersion, HardwareUid, MessageStatus, NewValues, NrOfSupportedComponents,
    OpenDeckRequest, OpenDeckResponse, SpecialRequest, SpecialResponse, ValueSize, Wish,
};
use midi_types::{Value14, Value7};

const OPENDECK_UID: u32 = 0x12345677;
const OPENDECK_ANALOGS: usize = 2;
const OPENDECK_ENCODERS: usize = 2;
const OPENDECK_LEDS: usize = 8;
const OPENDECK_BUTTONS: usize = 8;
const OPENDECK_NR_PRESETS: usize = 2;
const OPENDECK_MAX_NR_MESSAGES: usize = 2;

use heapless::Vec;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Preset {
    buttons: Vec<Button, OPENDECK_BUTTONS>,
    encoders: Vec<Encoder, OPENDECK_ENCODERS>,
    analogs: Vec<Analog, OPENDECK_ANALOGS>,
}

impl Default for Preset {
    fn default() -> Self {
        let mut buttons = Vec::new();
        for i in 0..OPENDECK_BUTTONS {
            buttons.push(Button::new(Value7::new(i as u8))).unwrap();
        }
        let mut encoders = Vec::new();
        for i in 0..OPENDECK_ENCODERS {
            encoders
                .push(Encoder::new(Value14::new(i16::MIN + i as i16)))
                .unwrap();
        }
        let mut analogs = Vec::new();
        for i in 0..OPENDECK_ANALOGS {
            analogs
                .push(Analog::new(Value14::new(i16::MIN + i as i16)))
                .unwrap();
        }

        Preset {
            buttons,
            encoders,
            analogs,
        }
    }
}

impl Preset {
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
}

#[derive(Default)]
pub struct Config {
    enabled: bool,
    current_preset: usize,
    presets: Vec<Preset, OPENDECK_NR_PRESETS>,
}

type Responses = Vec<Buffer, OPENDECK_MAX_NR_MESSAGES>;

impl Config {
    pub fn new() -> Self {
        let mut presets = Vec::new();
        for _ in 0..OPENDECK_NR_PRESETS {
            presets.push(Preset::default()).unwrap();
        }

        Config {
            enabled: false,
            current_preset: 0,
            presets,
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
                // FIXME callback
                //                rp2040_hal::rom_data::reset_to_usb_boot(0, 0);
                None
            }
            SpecialRequest::Reboot => {
                // FIXME callback
                //              cortex_m::peripheral::SCB::sys_reset();
                None
            }
            SpecialRequest::Handshake => {
                self.enabled = true;
                Some(SpecialResponse::Handshake)
            }
            SpecialRequest::ValueSize => Some(SpecialResponse::ValueSize),
            SpecialRequest::ValuesPerMessage => Some(SpecialResponse::ValuesPerMessage(32)),
            SpecialRequest::FirmwareVersion => {
                Some(SpecialResponse::FirmwareVersion(firmware_version()))
            }
            SpecialRequest::HardwareUID => {
                Some(SpecialResponse::HardwareUID(HardwareUid(OPENDECK_UID)))
            }
            SpecialRequest::FirmwareVersionAndHardwareUUID => {
                Some(SpecialResponse::FirmwareVersionAndHardwareUUID(
                    firmware_version(),
                    HardwareUid(OPENDECK_UID),
                ))
            }
            SpecialRequest::BootloaderSupport => Some(SpecialResponse::BootloaderSupport(true)),
            SpecialRequest::NrOfSupportedPresets => {
                Some(SpecialResponse::NrOfSupportedPresets(OPENDECK_NR_PRESETS))
            }
            SpecialRequest::NrOfSupportedComponents => Some(
                SpecialResponse::NrOfSupportedComponents(NrOfSupportedComponents {
                    buttons: OPENDECK_BUTTONS,
                    encoders: OPENDECK_ENCODERS,
                    analog: OPENDECK_ANALOGS,
                    leds: OPENDECK_LEDS,
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
                Block::Global(GlobalSection::Presets(pi, value)) => {
                    match pi {
                        PresetIndex::Active => match wish {
                            Wish::Set => self.current_preset = *value as usize,
                            Wish::Get | Wish::Backup => {
                                res_values.push(self.current_preset as u16).unwrap()
                            }
                        },
                        // FIXME implement more preset features
                        PresetIndex::Preservation => {}
                        PresetIndex::EnableMideChange => {}
                        PresetIndex::ForceValueRefresh => {}
                    }
                }
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

                Block::Display => {}
                Block::Led => {}
                Block::Touchscreen => {}
            };
        };

        (res_values, for_amount)
    }

    fn current_preset_mut(&mut self) -> Option<&mut Preset> {
        self.presets.get_mut(self.current_preset)
    }
}

fn firmware_version() -> FirmwareVersion {
    FirmwareVersion {
        major: 1,
        minor: 0,
        revision: 0,
    }
}