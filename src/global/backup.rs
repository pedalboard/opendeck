use crate::global::{GlobalPreset, GlobalSection, PresetIndex};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct GlobalPresetBackupIterator {
    preset_index: PresetIndex,
    done: bool,
}

impl GlobalPresetBackupIterator {
    pub fn new() -> Self {
        GlobalPresetBackupIterator {
            preset_index: PresetIndex::Active,
            done: false,
        }
    }
    pub fn next(&mut self, global_preset: &GlobalPreset) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let key = self.preset_index;
        let value = match self.preset_index {
            PresetIndex::Active => {
                self.preset_index = PresetIndex::Preservation;
                global_preset.current as u16
            }
            PresetIndex::Preservation => {
                self.preset_index = PresetIndex::ForceValueRefresh;
                global_preset.preserve_preset as u16
            }
            PresetIndex::ForceValueRefresh => {
                self.preset_index = PresetIndex::EnableMidiChange;
                global_preset.force_value_refresh as u16
            }
            PresetIndex::EnableMidiChange => {
                self.done = true;
                global_preset.force_value_refresh as u16
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Global(GlobalSection::Presets(key, value)),
            new_values,
        ))
    }
}

impl Default for GlobalPresetBackupIterator {
    fn default() -> Self {
        Self::new()
    }
}
