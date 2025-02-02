use frugger_core::FrugInputs;
use heapless::LinearMap;
use crate::OneBit;

type UpdateFn<C> = fn(&mut C, &FrugInputs, &mut OneBit) -> usize;

/// State machine.
pub struct SM<C> {
    curr: usize,
    states: LinearMap<usize, UpdateFn<C>, 100>,
}

impl<C> SM<C> {
    pub fn new() -> Self {
        Self {
            states: LinearMap::new(),
            curr: 0,
        }
    }

    pub fn add(&mut self, id: usize, state: UpdateFn<C>) {
        self.states.insert(id, state);
    }

    pub fn tick(&mut self, state: &mut C, inputs: &FrugInputs, engine: &mut OneBit) {
        self.curr = if let Some(func) = self.states.get(&self.curr) {
            func(state, inputs, engine)
        } else {
            self.curr
        }
    }
}
