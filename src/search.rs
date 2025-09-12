use std::{
    cmp::Ordering,
    sync::{Arc, RwLock, mpsc::channel},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use vampirc_uci::UciTimeControl;

use crate::{
    board::{Board, Color, Piece, PieceKind},
    r#move::Move,
    move_generator::Bitops,
    uci::Status,
};

impl Board {
    const WHITE_PAWN_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, -20, -20, 10, 10, 5, 5, -5, -10, 0, 0, -10, -5, 5, 0, 0,
        0, 20, 20, 0, 0, 0, 5, 5, 10, 25, 25, 10, 5, 5, 10, 10, 20, 30, 30, 20, 10, 10, 50, 50, 50,
        50, 50, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const BLACK_PAWN_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const KNIGHT_SQUARE_TABLE: [i64; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15,
        10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15,
        15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ];
    const WHITE_BISHOP_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 5, 10,
        10, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const BLACK_BISHOP_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];
    const WHITE_ROOK_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
        0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5, 10, 10, 10, 10, 10, 10, 5,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const BLACK_ROOK_SQUARE_TABLE: [i64; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ];
    const WHITE_QUEEN_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0,
        -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0,
        0, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];
    const BLACK_QUEEN_SQUARE_TABLE: [i64; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0,
        -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0,
        5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];
    const WHITE_KING_SQUARE_TABLE: [i64; 64] = [
        20, 30, 10, 0, 0, 10, 30, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20,
        -20, -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30,
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30,
    ];
    const BLACK_KING_SQUARE_TABLE: [i64; 64] = [
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ];

    fn material_scores(&self) -> i64 {
        let mut score = 0;
        for square in 0..64 {
            score += self.get_piece(square).score();
        }
        score
    }

    fn square_table_scores(&self) -> i64 {
        let mut score = 0;
        for square in 0..64 {
            let piece = self.get_piece(square);
            if piece != Piece::NONE {
                score += match piece {
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Pawn,
                    } => Self::WHITE_PAWN_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Pawn,
                    } => Self::BLACK_PAWN_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Rook,
                    } => Self::WHITE_ROOK_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Rook,
                    } => Self::BLACK_ROOK_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Knight,
                    } => Self::KNIGHT_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Knight,
                    } => Self::KNIGHT_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Bishop,
                    } => Self::WHITE_BISHOP_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Bishop,
                    } => Self::BLACK_BISHOP_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::Queen,
                    } => Self::WHITE_QUEEN_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::Queen,
                    } => Self::BLACK_QUEEN_SQUARE_TABLE,
                    Piece {
                        color: Color::White,
                        kind: PieceKind::King,
                    } => Self::WHITE_KING_SQUARE_TABLE,
                    Piece {
                        color: Color::Black,
                        kind: PieceKind::King,
                    } => Self::BLACK_KING_SQUARE_TABLE,
                    _ => unreachable!(),
                }[square as usize]
                    * match self.turn {
                        Color::White => 1,
                        Color::Black => -1,
                        Color::None => unreachable!(),
                    };
            }
        }
        score
    }

    fn eval(&mut self) -> i64 {
        let mut score = 0;
        score += self.material_scores();
        score += self.square_table_scores();
        score
            * match self.turn {
                Color::White => 1,
                Color::Black => -1,
                Color::None => unreachable!(),
            }
    }
}

#[derive(Debug)]
pub struct Search {
    pub best_move: Move,
    depth: usize,

    nodes: usize,

    score: i64,

    start: Instant,

    stopper: Arc<RwLock<Status>>,
}

impl Search {
    const BIG_NUM: i64 = 10000000;

    fn new(stopper: Arc<RwLock<Status>>) -> Self {
        Self {
            best_move: Move::default(),
            depth: 0,

            nodes: 0,

            score: 0,

            start: Instant::now(),

            stopper,
        }
    }

    pub fn go(
        board: &mut Board,
        time_control: Option<UciTimeControl>,
        stopper: Arc<RwLock<Status>>,
    ) -> Search {
        let (alpha, beta) = (-i64::MAX, i64::MAX);
        let mut search = Self::new(stopper.clone());
        if let Some(time_control) = time_control {
            let move_time = match time_control {
                UciTimeControl::TimeLeft {
                    white_time: Some(white_time),
                    black_time: Some(black_time),
                    ..
                } => match board.turn {
                    Color::White => white_time.num_milliseconds() / 20,
                    Color::Black => black_time.num_milliseconds() / 20,
                    Color::None => unreachable!(),
                },
                _ => 0,
            };
            if move_time != 0 {
                let stopper = stopper.clone();
                let (sender, receiver) = channel();
                thread::spawn(move || {
                    sleep(Duration::from_millis(move_time as u64));
                    if receiver.try_recv().is_err() {
                        *stopper.write().unwrap() = Status::Stopping;
                    }
                });
                let mut depth = 1;
                while *search.stopper.read().unwrap() != Status::Stopping && depth < 50 {
                    search.depth = depth;
                    search.score = search.negamax(board, search.depth, alpha, beta);
                    println!(
                        "info depth {} score cp {} nodes {} nps {} pv {}",
                        search.depth,
                        search.score * -1,
                        search.nodes,
                        (search.nodes as f64 / search.start.elapsed().as_secs_f64()) as u64,
                        search.best_move,
                    );
                    depth += 1;
                }

                let _ = sender.send(());
                return search;
            }
        }
        let max_depth = 4;
        for i in 1..=max_depth {
            search.depth = i;
            search.negamax(board, search.depth, alpha, beta);
        }
        search
    }

    fn mvv_lva(&mut self, board: &Board, a: Move, b: Move) -> Ordering {
        if a == self.best_move {
            Ordering::Less
        } else if b == self.best_move {
            Ordering::Greater
        } else if a.is_capture() && !b.is_capture() {
            Ordering::Less
        } else if !a.is_capture() && b.is_capture() {
            Ordering::Greater
        } else if a.is_capture() && b.is_capture() {
            if board.get_piece(a.to()).value() > board.get_piece(b.to()).value() {
                Ordering::Less
            } else if board.get_piece(a.to()).value() < board.get_piece(b.to()).value() {
                Ordering::Greater
            } else {
                board
                    .get_piece(a.from())
                    .value()
                    .cmp(&board.get_piece(b.from()).value())
            }
        } else {
            Ordering::Equal
        }
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i64, beta: i64) -> i64 {
        self.nodes += 1;
        if *self.stopper.read().unwrap() == Status::Stopping {
            return Self::BIG_NUM;
        }
        let mut best = board.eval();
        // Stand Pat
        if best >= beta {
            return best;
        }
        if best > alpha {
            alpha = best;
        }

        let mut moves = board
            .generate_moves()
            .iter()
            .filter(|e| e.is_capture())
            .cloned()
            .collect::<Vec<_>>();

        moves.sort_by(|a, b| self.mvv_lva(board, *a, *b));
        for m in moves {
            if !m.is_capture() {
                continue;
            }
            board.make_move(m);
            let score = -self.quiescence_search(board, -beta, -alpha);
            board.unmake_move(m);
            if score > best {
                best = score;
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

    fn negamax(&mut self, board: &mut Board, depth: usize, mut alpha: i64, beta: i64) -> i64 {
        self.nodes += 1;
        if *self.stopper.read().unwrap() == Status::Stopping {
            return Self::BIG_NUM;
        }
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta);
        }
        let mut best = -i64::MAX;
        let mut moves = board.generate_moves();
        moves.sort_by(|a, b| self.mvv_lva(board, *a, *b));
        for m in moves {
            board.make_move(m);
            let score = -self.negamax(board, depth - 1, -beta, -alpha);
            board.unmake_move(m);
            if score > best {
                best = score;
                if depth == self.depth {
                    self.best_move = m;
                }
                if score > alpha {
                    alpha = score;
                }
            }
            if score >= beta {
                break;
            }
        }

        if best == -Self::BIG_NUM {
            board.change_turn();
            let king_under_attack =
                board.king_under_attack((board.opponent_pieces() & board.kings).pop_lsb().unwrap());
            board.change_turn();
            if king_under_attack {
                -Search::BIG_NUM + (self.depth - depth) as i64
            } else {
                500
            }
        } else {
            best
        }
    }
}
