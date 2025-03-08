use crate::analog::handler::AnalogMessages;
use crate::button::handler::ButtonMessages;

pub enum Messages<'a> {
    Button(ButtonMessages<'a>),
    Analog(AnalogMessages<'a>),
}
