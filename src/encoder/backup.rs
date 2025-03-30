use crate::encoder::{Encoder, EncoderSection, EncoderSectionId};
use crate::{Amount, Block, NewValues, OpenDeckResponse, Wish};

pub struct EncoderBackupIterator {
    index: u16,
    section_id: EncoderSectionId,
    done: bool,
}

impl EncoderBackupIterator {
    pub fn new(index: usize) -> Self {
        EncoderBackupIterator {
            index: index as u16,
            section_id: EncoderSectionId::Enabled,
            done: false,
        }
    }
    pub fn next(&mut self, encoder: &Encoder) -> Option<OpenDeckResponse> {
        if self.done {
            return None;
        }
        let new_values = NewValues::new();
        let section = match self.section_id {
            EncoderSectionId::Enabled => {
                self.section_id = EncoderSectionId::InvertState;
                EncoderSection::Enabled(encoder.enabled)
            }
            EncoderSectionId::InvertState => {
                self.section_id = EncoderSectionId::MessageType;
                EncoderSection::Inverted(encoder.inverted)
            }
            EncoderSectionId::MessageType => {
                self.section_id = EncoderSectionId::MidiIdLSB;
                EncoderSection::MessageType(encoder.message_type)
            }
            EncoderSectionId::MidiIdLSB => {
                self.section_id = EncoderSectionId::Channel;
                EncoderSection::MidiIdLSB(encoder.midi_id)
            }
            EncoderSectionId::Channel => {
                self.section_id = EncoderSectionId::PulsesPerStep;
                EncoderSection::Channel(encoder.channel)
            }
            EncoderSectionId::PulsesPerStep => {
                self.section_id = EncoderSectionId::Accelleration;
                EncoderSection::PulsesPerStep(encoder.pulses_per_step)
            }
            EncoderSectionId::Accelleration => {
                self.section_id = EncoderSectionId::RemoteSync;
                EncoderSection::Accelleration(encoder.accelleration)
            }
            EncoderSectionId::RemoteSync => {
                self.section_id = EncoderSectionId::LowerLimit;
                EncoderSection::RemoteSync(encoder.remote_sync)
            }
            EncoderSectionId::LowerLimit => {
                self.section_id = EncoderSectionId::UpperLimit;
                EncoderSection::LowerLimit(encoder.lower_limit)
            }
            EncoderSectionId::UpperLimit => {
                self.section_id = EncoderSectionId::RepeatedValue;
                EncoderSection::UpperLimit(encoder.upper_limit)
            }
            EncoderSectionId::RepeatedValue => {
                self.section_id = EncoderSectionId::SecondMidiId;
                EncoderSection::RepeatedValue(encoder.value)
            }
            EncoderSectionId::SecondMidiId => {
                self.done = true;
                EncoderSection::SecondMidiId(encoder.second_midi_id)
            }
            EncoderSectionId::MidiIdMSB => {
                self.done = true;
                EncoderSection::MidiIdMSB((encoder.midi_id >> 7) as u8)
            }
        };

        Some(OpenDeckResponse::Configuration(
            Wish::Set,
            Amount::Single,
            Block::Encoder(self.index, section),
            new_values,
        ))
    }
}
