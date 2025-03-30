use crate::{
    button::backup::ButtonBackupIterator,
    config::{Config, Preset},
    encoder::backup::EncoderBackupIterator,
    global::{GlobalSection, PresetIndex},
    Amount, Block, NewValues, OpenDeckResponse, SpecialResponse, Wish,
};

enum BackupStatus {
    Init,
    PresetChange,
    Running,
    Done,
}

pub struct ConfigBackupIterator<
    const P: usize,
    const B: usize,
    const A: usize,
    const E: usize,
    const L: usize,
> {
    preset: usize,
    presets: PresetBackupIterator<B, A, E, L>,
    status: BackupStatus,
}

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    ConfigBackupIterator<P, B, A, E, L>
{
    pub fn new() -> Self {
        ConfigBackupIterator {
            preset: 0,
            presets: PresetBackupIterator::new(),
            status: BackupStatus::Init,
        }
    }
    pub fn next(&mut self, config: &mut Config<P, B, A, E, L>) -> Option<OpenDeckResponse> {
        match self.status {
            BackupStatus::Init => {
                self.status = BackupStatus::PresetChange;
                Some(OpenDeckResponse::Special(SpecialResponse::Backup))
            }
            BackupStatus::PresetChange => {
                self.status = BackupStatus::Running;
                Some(OpenDeckResponse::Configuration(
                    Wish::Set,
                    Amount::Single,
                    Block::Global(GlobalSection::Presets(
                        PresetIndex::Active,
                        self.preset as u16,
                    )),
                    NewValues::new(),
                ))
            }
            BackupStatus::Running => {
                let res = self.presets.next(&config.presets[self.preset]);
                if res.is_none() {
                    self.preset += 1;
                    if self.preset >= P {
                        self.status = BackupStatus::Done;
                        return Some(OpenDeckResponse::Special(SpecialResponse::Backup));
                    }
                    self.presets = PresetBackupIterator::new();
                    return Some(OpenDeckResponse::Configuration(
                        Wish::Set,
                        Amount::Single,
                        Block::Global(GlobalSection::Presets(
                            PresetIndex::Active,
                            self.preset as u16,
                        )),
                        NewValues::new(),
                    ));
                }
                res
            }
            BackupStatus::Done => None,
        }
    }
}
impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize> Default
    for ConfigBackupIterator<P, B, A, E, L>
{
    fn default() -> Self {
        ConfigBackupIterator::new()
    }
}

pub struct PresetBackupIterator<const B: usize, const A: usize, const E: usize, const L: usize> {
    button_index: usize,
    button_iter: ButtonBackupIterator,
    encoder_index: usize,
    encoder_iter: EncoderBackupIterator,
}

impl<const B: usize, const A: usize, const E: usize, const L: usize>
    PresetBackupIterator<B, A, E, L>
{
    fn new() -> Self {
        // Implementation of the iterator logic goes here
        PresetBackupIterator {
            button_index: 0,
            button_iter: ButtonBackupIterator::new(0),
            encoder_index: 0,
            encoder_iter: EncoderBackupIterator::new(0),
        }
    }
    fn next(&mut self, preset: &Preset<B, A, E, L>) -> Option<OpenDeckResponse> {
        if self.button_index < B {
            let res = self.button_iter.next(&preset.buttons[self.button_index]);
            if res.is_some() {
                return res;
            }
            self.button_index += 1;
            if self.button_index < B {
                self.button_iter = ButtonBackupIterator::new(self.button_index);
                return self.button_iter.next(&preset.buttons[self.button_index]);
            }
        }
        if self.encoder_index < E {
            let res = self.encoder_iter.next(&preset.encoders[self.encoder_index]);
            if res.is_some() {
                return res;
            }
            self.encoder_index += 1;
            if self.encoder_index < E {
                self.encoder_iter = EncoderBackupIterator::new(self.encoder_index);
                return self.encoder_iter.next(&preset.encoders[self.encoder_index]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        button::{ButtonMessageType, ButtonSection, ButtonType},
        config::{Config, FirmwareVersion},
        encoder::{Accelleration, EncoderMessageType, EncoderSection},
        Amount, Block, ChannelOrAll, NewValues, Wish,
    };

    #[test]
    fn test_config_backup_iterator() {
        let version = FirmwareVersion {
            major: 1,
            minor: 0,
            revision: 0,
        };
        let uid = 12345;
        let reboot = || {};
        let bootloader = || {};
        let config = &mut Config::<1, 2, 1, 1, 1>::new(version, uid, reboot, bootloader);

        let mut iterator = ConfigBackupIterator::new();
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Special(SpecialResponse::Backup))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Global(GlobalSection::Presets(PresetIndex::Active, 0)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::Type(ButtonType::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::MessageType(ButtonMessageType::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::MidiId(0)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::Value(0x01)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::Channel(ChannelOrAll::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(1, ButtonSection::Type(ButtonType::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(1, ButtonSection::MessageType(ButtonMessageType::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(1, ButtonSection::MidiId(1)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(1, ButtonSection::Value(0x01)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(1, ButtonSection::Channel(ChannelOrAll::default())),
                NewValues::new(),
            ))
        );

        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::Enabled(false)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::Inverted(false)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(
                    0,
                    EncoderSection::MessageType(EncoderMessageType::default())
                ),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::MidiIdLSB(0)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::Channel(ChannelOrAll::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::PulsesPerStep(4)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::Accelleration(Accelleration::None)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::RemoteSync(false)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::LowerLimit(0)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::UpperLimit(127)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::RepeatedValue(0)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Encoder(0, EncoderSection::SecondMidiId(0)),
                NewValues::new(),
            ))
        );

        assert_eq!(
            iterator.next(config),
            Some(OpenDeckResponse::Special(SpecialResponse::Backup))
        );

        assert_eq!(iterator.next(config), None);
    }
}
