use bitfield_struct::bitfield;

#[bitfield(u16, hash = true, order = msb)]
#[derive(Eq, PartialEq)]
pub struct IOBits {
    #[bits(12)]
    _padding: u16,
    #[bits(1, default = false)]
    pub on: bool,
    #[bits(1, default = false)]
    pub interupt: bool,
    #[bits(1, default = false)]
    pub done: bool,
    #[bits(1, default = false)]
    pub busy: bool,
}
