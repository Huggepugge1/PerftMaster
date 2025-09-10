use crate::{
    board::{Board, Color},
    r#move::Move,
};

impl Board {
    pub fn eval(&mut self) -> f64 {
        let mut score = 0f64;
        for square in 0..64 {
            score += self.get_piece(square).score()
        }
        score
            * match self.turn {
                Color::White => 1f64,
                Color::Black => -1f64,
                Color::None => unreachable!(),
            }
    }

    pub fn search(&mut self) -> Move {
        self.negamax(3).1
    }

    fn negamax(&mut self, depth: usize) -> (f64, Move) {
        if depth == 0 {
            return (self.eval(), Move::NULL);
        }
        let mut best = (f64::MIN, Move::NULL);
        for m in self.generate_moves() {
            self.make_move(m);
            let eval = -self.negamax(depth - 1).0;
            self.unmake_move(m);
            if eval > best.0 {
                best.0 = eval;
                best.1 = m;
            }
        }

        best
    }
}
