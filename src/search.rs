use crate::{board::Board, r#move::Move};
use rand::{self, rng, seq::IndexedRandom};

impl Board {
    pub fn search(&mut self) -> Move {
        let mut rng = rng();
        *self.generate_moves().choose(&mut rng).unwrap()
    }
}
