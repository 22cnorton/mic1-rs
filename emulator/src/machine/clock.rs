use getset::{Getters, Setters};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Setters, Getters)]
#[getset(set = "pub", get = "pub")]
pub struct Clock {
    tick: usize,
    subtick: Subtick,
}

impl Clock {
    pub(super) fn pulse(&mut self) {
        self.subtick = self.subtick.next_tick();
        if self.subtick.is_load() {
            self.tick += 1;
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            tick: 1,
            subtick: Default::default(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Default, Clone, Copy, Hash, derive_more::IsVariant)]
pub enum Subtick {
    #[default]
    Load,
    Gate,
    Operation,
    Store,
}

impl Subtick {
    pub(super) const fn next_tick(&self) -> Self {
        match self {
            Subtick::Load => Self::Gate,
            Subtick::Gate => Self::Operation,
            Subtick::Operation => Self::Store,
            Subtick::Store => Self::Load,
        }
    }
}
