use crate::{
    button::backup::ButtonBackupIterator,
    config::{Config, Preset},
    OpenDeckResponse,
};

pub struct ConfigBackupIterator<
    'a,
    const P: usize,
    const B: usize,
    const A: usize,
    const E: usize,
    const L: usize,
> {
    config: &'a Config<P, B, A, E, L>,
    preset: usize,
    presets: PresetBackupIterator<'a, B, A, E, L>,
}

impl<const P: usize, const B: usize, const A: usize, const E: usize, const L: usize> Iterator
    for ConfigBackupIterator<'_, P, B, A, E, L>
{
    type Item = OpenDeckResponse;

    fn next(&mut self) -> Option<Self::Item> {
        // Implementation of the iterator logic goes here
        let res = self.presets.next();
        if res.is_none() {
            self.preset += 1;
            if self.preset >= P {
                return None;
            }
            self.presets = PresetBackupIterator::new(&self.config.presets[self.preset]);
            return self.presets.next();
        }
        res
    }
}

impl<'a, const P: usize, const B: usize, const A: usize, const E: usize, const L: usize>
    ConfigBackupIterator<'a, P, B, A, E, L>
{
    pub fn new(config: &'a Config<P, B, A, E, L>) -> Self {
        ConfigBackupIterator {
            config,
            preset: 0,
            presets: PresetBackupIterator::new(&config.presets[0]),
        }
    }
}

pub struct PresetBackupIterator<'a, const B: usize, const A: usize, const E: usize, const L: usize>
{
    preset: &'a Preset<B, A, E, L>,
    button_index: usize,
    button_iter: ButtonBackupIterator<'a>,
}

impl<const B: usize, const A: usize, const E: usize, const L: usize> Iterator
    for PresetBackupIterator<'_, B, A, E, L>
{
    type Item = OpenDeckResponse;

    fn next(&mut self) -> Option<Self::Item> {
        if self.button_index < B {
            let res = self.button_iter.next();
            if res.is_some() {
                return res;
            }
            self.button_index += 1;
            if self.button_index < B {
                self.button_iter = ButtonBackupIterator::new(
                    &self.preset.buttons[self.button_index],
                    self.button_index,
                );
            }
        }
        // Implementation of the iterator logic goes here
        None
    }
}

impl<'a, const B: usize, const A: usize, const E: usize, const L: usize>
    PresetBackupIterator<'a, B, A, E, L>
{
    fn new(preset: &'a Preset<B, A, E, L>) -> Self {
        // Implementation of the iterator logic goes here
        PresetBackupIterator {
            preset,
            button_index: 0,
            button_iter: ButtonBackupIterator::new(&preset.buttons[0], 0),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        button::{ButtonMessageType, ButtonSection, ButtonType},
        config::{Config, FirmwareVersion},
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
        let config = &mut Config::<1, 1, 1, 1, 1>::new(version, uid, reboot, bootloader);

        let mut iterator = ConfigBackupIterator::new(config);
        assert_eq!(
            iterator.next(),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::Type(ButtonType::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::MessageType(ButtonMessageType::default())),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::MidiId(0)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::Value(0x01)),
                NewValues::new(),
            ))
        );
        assert_eq!(
            iterator.next(),
            Some(OpenDeckResponse::Configuration(
                Wish::Set,
                Amount::Single,
                Block::Button(0, ButtonSection::Channel(ChannelOrAll::default())),
                NewValues::new(),
            ))
        );
    }
}
