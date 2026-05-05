use crate::memory::traits::IOBitsType;
use bitfield_struct::bitfield;

#[bitfield(u16, hash = true, order = msb)]
#[derive(Eq, PartialEq)]
pub struct MoloneyIOBits {
    #[bits(12)]
    _padding: u16,
    #[bits(1, default = false)]
    on: bool,
    #[bits(1, default = false)]
    interupt: bool,
    #[bits(1, default = false)]
    done: bool,
    #[bits(1, default = false)]
    busy: bool,
}

impl IOBitsType<u16> for MoloneyIOBits {
    // type ValueType = u16;

    fn on(&self) -> bool {
        self.on()
    }
    fn done(&self) -> bool {
        self.done()
    }
    fn busy(&self) -> bool {
        self.busy()
    }
    fn interupt(&self) -> bool {
        self.interupt()
    }

    fn with_on(self, value: bool) -> Self {
        self.with_on(value)
    }
    fn with_done(self, value: bool) -> Self {
        self.with_done(value)
    }
    fn with_busy(self, value: bool) -> Self {
        self.with_busy(value)
    }
    fn with_interupt(self, value: bool) -> Self {
        self.with_interupt(value)
    }
}
