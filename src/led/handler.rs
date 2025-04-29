use crate::led::Led;
use midi2::{error::BufferOverflow, BytesMessage};

impl Led {
    fn process(m: BytesMessage<&[u8]>) {}
}
