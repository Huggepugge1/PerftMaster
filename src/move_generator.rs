use crate::{
    board::{Bitmap, Board, Color, Square},
    r#move::Move,
};

pub trait Bitops {
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
            .iter()
            .filter(|m| self.is_legal_move(**m))
            .copied()
            .collect()
    }

    fn is_legal_move(&mut self, m: Move) -> bool {
        self.make_move(m);

        let own = self.opponent_pieces();

        let is_legal = !self.king_under_attack((self.kings & own).pop_lsb().unwrap());

        self.unmake_move(m);
        is_legal
    }

    fn generate_pawn_moves(&mut self) -> Vec<Move> {
        match self.turn {
            Color::White => self.generate_white_pawn_moves(),
            Color::Black => self.generate_black_pawn_moves(),
            Color::None => unreachable!(),
        }
    }

    fn generate_white_pawn_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pawns = self.pawns & self.white_pieces;
        let blockers = self.white_pieces | self.black_pieces;
        let free = !blockers;
        while let Some(from) = pawns.pop_lsb() {
            if (1 << (from + 8)) & free > 0 {
                moves.append(&mut Move::add_promotion_if_possible(from, from + 8, 0b0000));
            }
            if (1 << (from + 7)) & self.black_pieces & NOT_H_FILE > 0 {
                moves.append(&mut Move::add_promotion_if_possible(from, from + 7, 0b0100));
            }
            if Bitmap::checked_shl(1, (from + 9) as u32).unwrap_or(0)
                & self.black_pieces
                & NOT_A_FILE
                > 0
            {
                moves.append(&mut Move::add_promotion_if_possible(from, from + 9, 0b0100));
            }
            if from / 8 == 1 && ((1 << (from + 8)) | (1 << (from + 16))) & blockers == 0 {
                moves.push(Move::new(from, from + 16, 0b0001));
            }
            if self.ep != -1 {
                if (1 << (from + 7)) & (1 << self.ep) & NOT_H_FILE > 0 {
                    moves.push(Move::new(from, from + 7, 0b0101));
                }
                if Bitmap::checked_shl(1, (from + 9) as u32).unwrap_or(0)
                    & (1 << self.ep)
                    & NOT_A_FILE
                    > 0
                {
                    moves.push(Move::new(from, from + 9, 0b0101));
                }
            }
        }

        moves
    }

    fn white_pawn_attacks(&self, from: i16) -> u64 {
        (Bitmap::checked_shl(1, (from + 7) as u32).unwrap_or(0) & NOT_H_FILE)
            | (Bitmap::checked_shl(1, (from + 9) as u32).unwrap_or(0) & NOT_A_FILE)
    }

    fn generate_black_pawn_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pawns = self.pawns & self.black_pieces;
        let blockers = self.white_pieces | self.black_pieces;
        let free = !blockers;
        while let Some(from) = pawns.pop_lsb() {
            if (1 << (from - 8)) & free > 0 {
                moves.append(&mut Move::add_promotion_if_possible(from, from - 8, 0b0000));
            }
            if (1 << (from - 7)) & self.white_pieces & NOT_A_FILE > 0 {
                moves.append(&mut Move::add_promotion_if_possible(from, from - 7, 0b0100));
            }
            if Bitmap::checked_shl(1, (from - 9) as u32).unwrap_or(0)
                & self.white_pieces
                & NOT_H_FILE
                > 0
            {
                moves.append(&mut Move::add_promotion_if_possible(from, from - 9, 0b0100));
            }
            if from / 8 == 6 && ((1 << (from - 8)) | (1 << (from - 16))) & blockers == 0 {
                moves.push(Move::new(from, from - 16, 0b0001));
            }
            if self.ep != -1 {
                if (1 << (from - 7)) & (1 << (self.ep)) & NOT_A_FILE > 0 {
                    moves.push(Move::new(from, from - 7, 0b0101));
                }
                if Bitmap::checked_shl(1, (from - 9) as u32).unwrap_or(0)
                    & (1 << self.ep)
                    & NOT_H_FILE
                    > 0
                {
                    moves.push(Move::new(from, from - 9, 0b0101));
                }
            }
        }
        moves
    }

    fn black_pawn_attacks(&self, from: i16) -> u64 {
        (Bitmap::checked_shl(1, (from - 7) as u32).unwrap_or(0) & NOT_A_FILE)
            | (Bitmap::checked_shl(1, (from - 9) as u32).unwrap_or(0) & NOT_H_FILE)
    }

    fn generate_rook_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = self.own_pieces();
        let opponent = self.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut rooks = self.rooks & own;
        while let Some(from) = rooks.pop_lsb() {
            let mut bitmap = self.rook_attacks(from, occupied) & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 {
                    0b0100
                } else {
                    0b0000
                };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    fn rook_attacks(&self, from: i16, occupied: u64) -> u64 {
        get_positive_ray_attacks(from, 1, occupied)
            | get_positive_ray_attacks(from, 8, occupied)
            | get_negative_ray_attacks(from, -1, occupied)
            | get_negative_ray_attacks(from, -8, occupied)
    }

    fn generate_knight_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = self.own_pieces();
        let opponent = self.opponent_pieces();

        let free = !own;

        let mut knights = self.knights & own;

        while let Some(from) = knights.pop_lsb() {
            let mut bitmap = self.knight_attacks(from) & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    fn knight_attacks(&self, from: i16) -> u64 {
        (Bitmap::checked_shl(1, (from + 15) as u32).unwrap_or(0) & NOT_H_FILE)
            | (Bitmap::checked_shl(1, (from + 17) as u32).unwrap_or(0) & NOT_A_FILE)
            | (Bitmap::checked_shl(1, (from + 6) as u32).unwrap_or(0) & NOT_GH_FILE)
            | (Bitmap::checked_shl(1, (from + 10) as u32).unwrap_or(0) & NOT_AB_FILE)
            | (Bitmap::checked_shl(1, (from - 10) as u32).unwrap_or(0) & NOT_GH_FILE)
            | (Bitmap::checked_shl(1, (from - 6) as u32).unwrap_or(0) & NOT_AB_FILE)
            | (Bitmap::checked_shl(1, (from - 17) as u32).unwrap_or(0) & NOT_H_FILE)
            | (Bitmap::checked_shl(1, (from - 15) as u32).unwrap_or(0) & NOT_A_FILE)
    }

    fn generate_bishop_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = self.own_pieces();
        let opponent = self.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut bishops = self.bishops & own;
        while let Some(from) = bishops.pop_lsb() {
            let mut bitmap = self.bishop_attacks(from, occupied) & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    fn bishop_attacks(&self, from: i16, occupied: u64) -> u64 {
        get_positive_ray_attacks(from, 7, occupied)
            | get_positive_ray_attacks(from, 9, occupied)
            | get_negative_ray_attacks(from, -7, occupied)
            | get_negative_ray_attacks(from, -9, occupied)
    }

    fn generate_queen_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = self.own_pieces();
        let opponent = self.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut queens = self.queens & own;
        while let Some(from) = queens.pop_lsb() {
            let mut bitmap = self.queen_attacks(from, occupied) & free;

            while let Some(to) = bitmap.pop_lsb() {
                let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }

        moves
    }

    fn queen_attacks(&self, from: i16, occupied: u64) -> u64 {
        get_positive_ray_attacks(from, 1, occupied)
            | get_positive_ray_attacks(from, 7, occupied)
            | get_positive_ray_attacks(from, 8, occupied)
            | get_positive_ray_attacks(from, 9, occupied)
            | get_negative_ray_attacks(from, -1, occupied)
            | get_negative_ray_attacks(from, -7, occupied)
            | get_negative_ray_attacks(from, -8, occupied)
            | get_negative_ray_attacks(from, -9, occupied)
    }

    fn generate_king_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let own = self.own_pieces();
        let opponent = self.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut king = self.kings & own;
        let from = king.pop_lsb().expect("No king found");

        let mut bitmap = self.king_attacks(from) & free;

        while let Some(to) = bitmap.pop_lsb() {
            let flags = if (1 << to) & opponent > 0 { 0b0100 } else { 0 };
            moves.push(Move::new(from, to, flags));
        }

        // Castling
        self.change_turn();

        match self.turn {
            Color::White => {
                if self.castling_rights & 0b0010 > 0
                    && occupied & (0b01100000 << 56) == 0
                    && !self.king_under_attack(60)
                    && !self.king_under_attack(61)
                {
                    moves.push(Move::new(from, from + 2, 0b0010));
                }
                if self.castling_rights & 0b0001 > 0
                    && occupied & (0b00001110 << 56) == 0
                    && !self.king_under_attack(60)
                    && !self.king_under_attack(59)
                {
                    moves.push(Move::new(from, from - 2, 0b0011));
                }
            }
            Color::Black => {
                if self.castling_rights & 0b1000 > 0
                    && occupied & 0b01100000 == 0
                    && !self.king_under_attack(4)
                    && !self.king_under_attack(5)
                {
                    moves.push(Move::new(from, from + 2, 0b0010));
                }
                if self.castling_rights & 0b0100 > 0
                    && occupied & 0b00001110 == 0
                    && !self.king_under_attack(4)
                    && !self.king_under_attack(3)
                {
                    moves.push(Move::new(from, from - 2, 0b0011));
                }
            }
            Color::None => unreachable!(),
        }

        self.change_turn();

        moves
    }

    fn king_attacks(&self, from: i16) -> u64 {
        let mut bitmap = (Bitmap::checked_shl(1, (from - 1) as u32).unwrap_or(0) & NOT_H_FILE)
            | Bitmap::checked_shl(1, from as u32).unwrap_or(0)
            | (Bitmap::checked_shl(1, (from + 1) as u32).unwrap_or(0) & NOT_A_FILE);
        bitmap |= bitmap.checked_shl(8_u32).unwrap_or(0);
        bitmap |= bitmap.checked_shr(8_u32).unwrap_or(0);
        bitmap
    }

    pub fn king_under_attack(&self, king_pos: Square) -> bool {
        let own = self.opponent_pieces();
        let opponent = self.own_pieces();

        let occupied = own | opponent;

        let mut king = 1 << king_pos;
        let from = king.pop_lsb().expect("No king found");

        (match self.turn {
            Color::White => self.black_pawn_attacks(from) & self.pawns,
            Color::Black => self.white_pawn_attacks(from) & self.pawns,
            Color::None => unreachable!(),
        } | (self.rook_attacks(from, occupied) & self.rooks)
            | (self.knight_attacks(from) & self.knights)
            | (self.bishop_attacks(from, occupied) & self.bishops)
            | (self.queen_attacks(from, occupied) & self.queens)
            | (self.king_attacks(from) & self.kings))
            & opponent
            > 0
    }
}
