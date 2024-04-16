use super::Alias;

pub struct EventEmmiter<const S: usize, E> {
    alias: Alias<S>,
    events: [E; S],
}

impl<const S: usize, E> EventEmmiter<S, E> {
    pub fn new(dist: &[f64; S], events: [E; S]) -> Self {
        Self {
            alias: Alias::new(dist),
            events,
        }
    }

    pub fn generate(&self) -> &E {
        &self.events[self.alias.generate()]
    }
}

impl<const S: usize, E: Clone> EventEmmiter<S, E> {
    pub fn generate_owned(&self) -> E {
        self.events[self.alias.generate()].clone()
    }
}
