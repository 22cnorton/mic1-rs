#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Default)]
pub(super) struct Clock {
    tick: usize,
    subtick: Subtick,
}

impl Clock {
    pub(super) fn pulse(&mut self) {
        self.subtick = self.subtick.next_tick();
        if self.subtick == Subtick::Load {
            self.tick += 1;
        }
    }

    pub(super) fn tick(&self) -> usize {
        self.tick
    }
    pub(super) fn subtick(&self) -> Subtick {
        self.subtick
    }

    pub(super) fn set_subtick(&mut self, value: Subtick) {
        self.subtick = value;
    }
}

#[derive(Eq, PartialEq, Debug, Default, Clone, Copy, Hash)]
pub(super) enum Subtick {
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
