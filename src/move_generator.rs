use crate::{
    board::{Bitmap, Board, Color, Square},
    r#move::Move,
};

trait Bitops {
    fn bitscan_forward(self) -> Square;
    fn bitscan_reverse(self) -> Square;
    fn pop_lsb(&mut self) -> Option<Square>;
    #[allow(dead_code)]
    fn print_as_board(self);
}

impl Bitops for Bitmap {
    fn bitscan_forward(self) -> Square {
        let trailing_zeros = self.trailing_zeros();
        if trailing_zeros == 64 {
            return 0;
        }
        trailing_zeros as Square
    }

    fn bitscan_reverse(self) -> Square {
        let leading_zeros = self.leading_zeros();
        if leading_zeros == 64 {
            return 0;
        }
        63 - leading_zeros as Square
    }

    fn pop_lsb(&mut self) -> Option<Square> {
        let result = self.trailing_zeros() as Square;
        if result < 64 {
            *self ^= 1 << result;
            Some(result)
        } else {
            None
        }
    }

    fn print_as_board(self) {
        for i in 0..8 {
            println!("{:08b}", ((self >> ((7 - i) * 8)) as u8).reverse_bits());
        }
        println!();
    }
}

const NOT_A_FILE: Bitmap = 0xFEFEFEFEFEFEFEFE;
const NOT_AB_FILE: Bitmap = 0xFCFCFCFCFCFCFCFC;
const NOT_GH_FILE: Bitmap = 0x3F3F3F3F3F3F3F3F;
const NOT_H_FILE: Bitmap = 0x7F7F7F7F7F7F7F7F;

fn get_ray(mut current: Square, dir: Square) -> Bitmap {
    let mut result = 0;
    current += dir;
    if dir % 8 == 1 || dir % 8 == -7 {
        while (current + 8) % 8 > 0 && (0..64).contains(&current) {
            result |= 1 << current;
            current += dir;
        }
    } else if dir % 8 == 7 || dir % 8 == -1 {
        while (current + 8) % 8 < 7 && (0..64).contains(&current) {
            result |= 1 << current;
            current += dir;
        }
    } else {
        while (0..64).contains(&current) {
            result |= 1 << current;
            current += dir;
        }
    }
    result
}

fn get_positive_ray_attacks(mut square: Square, dir: Square, occupied: Bitmap) -> Bitmap {
    let mut attacks = get_ray(square, dir);
    let blocker = attacks & occupied;
    if blocker > 0 {
        square = blocker.bitscan_forward();
        attacks ^= get_ray(square, dir);
    }
    attacks
}

fn get_negative_ray_attacks(mut square: Square, dir: Square, occupied: Bitmap) -> Bitmap {
    let mut attacks = get_ray(square, dir);
    let blocker = attacks & occupied;
    if blocker > 0 {
        square = blocker.bitscan_reverse();
        attacks ^= get_ray(square, dir);
    }
    attacks
}

impl Board {
    pub fn generate_moves(&mut self) -> Vec<Move> {
        let mut moves = self.generate_pawn_moves();
        moves.append(&mut self.generate_rook_moves());
        moves.append(&mut self.generate_knight_moves());
        moves.append(&mut self.generate_bishop_moves());
        moves.append(&mut self.generate_queen_moves());
        moves.append(&mut self.generate_king_moves());
        moves
    }

    pub fn generate_pawn_moves(&mut self) -> Vec<Move> {
        match self.turn {
            Color::White => self.generate_white_pawn_moves(),
            Color::Black => self.generate_black_pawn_moves(),
            Color::None => unreachable!(),
        }
    }

    pub fn generate_white_pawn_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pawns = self.pawns & self.white_pieces;
        let blockers = self.white_pieces | self.black_pieces;
        let free = !blockers;
        while let Some(from) = pawns.pop_lsb() {
            if (1 << (from + 8)) & free > 0 {
                moves.push(Move::new(from, from + 8, 0b0000));
            }
            if (1 << (from + 7)) & self.black_pieces & NOT_H_FILE > 0 {
                moves.push(Move::new(from, from + 7, 0b0100));
            }
            if (1 << (from + 9)) & self.black_pieces & NOT_A_FILE > 0 {
                moves.push(Move::new(from, from + 9, 0b0100));
            }
            if from / 8 == 1 && ((1 << (from + 8)) | (1 << (from + 16))) & blockers == 0 {
                moves.push(Move::new(from, from + 16, 0b0001));
            }
            if self.ep != -1 {
                if (1 << (from + 7)) & (1 << self.ep) & NOT_H_FILE > 0 {
                    moves.push(Move::new(from, from + 7, 0b0101));
                }
                if (1 << (from + 9)) & (1 << self.ep) & NOT_A_FILE > 0 {
                    moves.push(Move::new(from, from + 9, 0b0101));
                }
            }
        }
        moves
    }

    pub fn generate_black_pawn_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pawns = self.pawns & self.black_pieces;
        let blockers = self.white_pieces | self.black_pieces;
        let free = !blockers;
        while let Some(from) = pawns.pop_lsb() {
            if (1 << (from - 8)) & free > 0 {
                moves.push(Move::new(from, from - 8, 0b0000));
            }
            if (1 << (from - 7)) & self.white_pieces & NOT_A_FILE > 0 {
                moves.push(Move::new(from, from - 7, 0b0100));
            }
            if (1 << (from - 9)) & self.white_pieces & NOT_H_FILE > 0 {
                moves.push(Move::new(from, from - 9, 0b0100));
            }
            if from / 8 == 6 && ((1 << (from - 8)) | (1 << (from - 16))) & blockers == 0 {
                moves.push(Move::new(from, from - 16, 0b0001));
            }
            if self.ep != -1 {
                if (1 << (from - 7)) & (1 << self.ep) & NOT_A_FILE > 0 {
                    moves.push(Move::new(from, from - 7, 0b0101));
                }
                if (1 << (from - 9)) & (1 << self.ep) & NOT_H_FILE > 0 {
                    moves.push(Move::new(from, from - 9, 0b0101));
                }
            }
        }
        moves
    }

    pub fn generate_rook_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::None => unreachable!(),
        };
        let opponent = match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::None => unreachable!(),
        };

        let occupied = own | opponent;

        let free = !own;

        let mut rooks = self.rooks & own;
        while let Some(from) = rooks.pop_lsb() {
            let mut bitmap = (get_positive_ray_attacks(from, 1, occupied)
                | get_positive_ray_attacks(from, 8, occupied)
                | get_negative_ray_attacks(from, -1, occupied)
                | get_negative_ray_attacks(from, -8, occupied))
                & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    pub fn generate_knight_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::None => unreachable!(),
        };
        let opponent = match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::None => unreachable!(),
        };

        let free = !own;

        let mut knights = self.knights & own;

        while let Some(from) = knights.pop_lsb() {
            let mut bitmap =
                (Bitmap::checked_shl(1, (from + 15) as u32).unwrap_or(0) & free & NOT_H_FILE)
                    | (Bitmap::checked_shl(1, (from + 17) as u32).unwrap_or(0) & NOT_A_FILE)
                    | (Bitmap::checked_shl(1, (from + 6) as u32).unwrap_or(0) & NOT_GH_FILE)
                    | (Bitmap::checked_shl(1, (from + 10) as u32).unwrap_or(0) & NOT_AB_FILE)
                    | (Bitmap::checked_shl(1, (from - 10) as u32).unwrap_or(0) & NOT_GH_FILE)
                    | (Bitmap::checked_shl(1, (from - 6) as u32).unwrap_or(0) & NOT_AB_FILE)
                    | (Bitmap::checked_shl(1, (from - 17) as u32).unwrap_or(0) & NOT_H_FILE)
                    | (Bitmap::checked_shl(1, (from - 15) as u32).unwrap_or(0) & NOT_A_FILE);

            bitmap &= free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    pub fn generate_bishop_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::None => unreachable!(),
        };
        let opponent = match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::None => unreachable!(),
        };

        let occupied = own | opponent;

        let free = !own;

        let mut bishops = self.bishops & own;
        while let Some(from) = bishops.pop_lsb() {
            let mut bitmap = (get_positive_ray_attacks(from, 7, occupied)
                | get_positive_ray_attacks(from, 9, occupied)
                | get_negative_ray_attacks(from, -7, occupied)
                | get_negative_ray_attacks(from, -9, occupied))
                & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    pub fn generate_queen_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::None => unreachable!(),
        };
        let opponent = match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::None => unreachable!(),
        };

        let occupied = own | opponent;

        let free = !own;

        let mut queens = self.queens & own;
        while let Some(from) = queens.pop_lsb() {
            let mut bitmap = (get_positive_ray_attacks(from, 1, occupied)
                | get_positive_ray_attacks(from, 7, occupied)
                | get_positive_ray_attacks(from, 8, occupied)
                | get_positive_ray_attacks(from, 9, occupied)
                | get_negative_ray_attacks(from, -1, occupied)
                | get_negative_ray_attacks(from, -7, occupied)
                | get_negative_ray_attacks(from, -8, occupied)
                | get_negative_ray_attacks(from, -9, occupied))
                & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    pub fn generate_king_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = match self.turn {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
            Color::None => unreachable!(),
        };
        let opponent = match self.turn {
            Color::White => self.black_pieces,
            Color::Black => self.white_pieces,
            Color::None => unreachable!(),
        };

        let occupied = own | opponent;

        let free = !own;

        let mut king = self.kings & own;
        let from = king.pop_lsb().expect("No king found");

        let mut bitmap = (Bitmap::checked_shl(1, (from - 1) as u32).unwrap_or(0) & NOT_H_FILE)
            | Bitmap::checked_shl(1, from as u32).unwrap_or(0)
            | (Bitmap::checked_shl(1, (from + 1) as u32).unwrap_or(0) & NOT_A_FILE);
        bitmap |= bitmap.checked_shl(8_u32).unwrap_or(0);
        bitmap |= bitmap.checked_shr(8_u32).unwrap_or(0);

        bitmap &= free;

        while let Some(to) = bitmap.pop_lsb() {
            let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
            moves.push(Move::new(from, to, flags));
        }

        // Castling
        match self.turn {
            Color::White => {
                if self.castling_rights & 0b1000 > 0 && occupied & 0b01100000 == 0 {
                    moves.push(Move::new(from, from + 2, 0b0010));
                }
                if self.castling_rights & 0b0100 > 0 && occupied & 0b00001110 == 0 {
                    moves.push(Move::new(from, from - 2, 0b0011));
                }
            }
            Color::Black => {
                if self.castling_rights & 0b0010 > 0 && occupied & (0b01100000 << 56) == 0 {
                    moves.push(Move::new(from, from + 2, 0b0010));
                }
                if self.castling_rights & 0b0001 > 0 && occupied & (0b00001110 << 56) == 0 {
                    moves.push(Move::new(from, from - 2, 0b0011));
                }
            }
            Color::None => unreachable!(),
        }

        moves
    }
}
