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
        let (alpha, beta) = (f64::MIN, f64::MAX);
        self.negamax(5, alpha, beta).1
    }

    fn negamax(&mut self, depth: usize, mut alpha: f64, beta: f64) -> (f64, Move) {
        if depth == 0 {
            return (self.eval(), Move::NULL);
        }
        let mut best = (f64::MIN, Move::NULL);
        for m in self.generate_moves() {
            self.make_move(m);
            let score = -self.negamax(depth - 1, -beta, -alpha).0;
            self.unmake_move(m);
            if score > best.0 {
                best.0 = score;
                best.1 = m;
                if score > alpha {
                    alpha = score;
                }
            }
            if score >= beta {
                break;
            }
        }

        best
    }
}
