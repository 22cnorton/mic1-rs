use bitfield_struct::bitfield;


#[bitfield(u16, hash = true, order = msb)]
#[derive(Eq, PartialEq)]
struct IOBits {
    #[bits(12)]
    _padding: u16,
    #[bits(1, default = false)]
    o: bool,
    #[bits(1, default = false)]
    i: bool,
    #[bits(1, default = false)]
    d: bool,
    #[bits(1, default = false)]
    b: bool,
}

