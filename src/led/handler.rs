use crate::led::Led;
use midi2::BytesMessage;

impl Led {
    pub fn process(_m: BytesMessage<&[u8]>) {}
}
