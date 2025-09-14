use crate::{
    board::{Bitboard, Board, Color, Square},
    r#move::Move,
};

pub trait Bitops {
    fn bitscan_forward(self) -> Option<Square>;
    fn bitscan_reverse(self) -> Option<Square>;
    fn pop_lsb(&mut self) -> Option<Square>;
    #[allow(dead_code)]
    fn print(self, title: &str);
}

impl Bitops for Bitboard {
    fn bitscan_forward(self) -> Option<Square> {
        let trailing_zeros = self.trailing_zeros();
        if trailing_zeros == 64 {
            None
        } else {
            Some(trailing_zeros as Square)
        }
    }

    fn bitscan_reverse(self) -> Option<Square> {
        let leading_zeros = self.leading_zeros();
        if leading_zeros == 64 {
            None
        } else {
            Some(63 - leading_zeros as Square)
        }
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

    fn print(self, title: &str) {
        println!("{title}");
        for i in 0..8 {
            println!("{:08b}", ((self >> ((7 - i) * 8)) as u8).reverse_bits());
        }
        println!();
    }
}

const NOT_A_FILE: Bitboard = 0xFEFEFEFEFEFEFEFE;
const NOT_AB_FILE: Bitboard = 0xFCFCFCFCFCFCFCFC;
const NOT_GH_FILE: Bitboard = 0x3F3F3F3F3F3F3F3F;
const NOT_H_FILE: Bitboard = 0x7F7F7F7F7F7F7F7F;

#[derive(Clone, Copy, Debug)]
enum Dir {
    North,
    NorthWest,
    NorthEast,
    South,
    SouthWest,
    SouthEast,
    West,
    East,
}

impl Dir {
    const CARDINALITY: usize = 8;
    const MEMBERS: [Dir; 8] = [
        Dir::North,
        Dir::NorthWest,
        Dir::NorthEast,
        Dir::South,
        Dir::SouthWest,
        Dir::SouthEast,
        Dir::West,
        Dir::East,
    ];

    const fn to_square(self) -> Square {
        match self {
            Dir::North => 8,
            Dir::NorthEast => 9,
            Dir::NorthWest => 7,
            Dir::East => 1,
            Dir::West => -1,
            Dir::South => -8,
            Dir::SouthWest => -9,
            Dir::SouthEast => -7,
        }
    }

    const fn from_squares(from: Square, to: Square) -> Option<Dir> {
        let files = to % 8 - from % 8;
        let ranks = to / 8 - from / 8;
        const fn max(a: Square, b: Square) -> Square {
            if a > b { a } else { b }
        }
        if files == 0 || ranks == 0 || files.abs() == ranks.abs() {
            let square_dir = (to - from)
                / if files == 0 {
                    max(ranks.abs(), 1)
                } else {
                    max(files.abs(), 1)
                };
            match square_dir {
                8 => return Some(Dir::North),
                9 => return Some(Dir::NorthEast),
                7 => return Some(Dir::NorthWest),
                1 => return Some(Dir::East),
                -1 => return Some(Dir::West),
                -8 => return Some(Dir::South),
                -9 => return Some(Dir::SouthWest),
                -7 => return Some(Dir::SouthEast),
                _ => None,
            }
        } else {
            None
        }
    }

    const fn rem(self, rhs: Square) -> Square {
        (self.to_square() + 16) % rhs
    }
}

static RAYS: [[Bitboard; 64]; 8] = generate_rays();
static IN_BETWEEN_RAYS: [[Bitboard; 64]; 64] = generate_in_between_rays();

const fn generate_rays() -> [[Bitboard; 64]; 8] {
    let mut rays = [[0; 64]; 8];

    let mut square = 0;
    while square < 64 {
        let mut dir = 0;
        while dir < Dir::CARDINALITY {
            rays[dir][square] = get_ray(square as Square, Dir::MEMBERS[dir]);
            dir += 1;
        }
        square += 1;
    }

    rays
}

const fn generate_in_between_rays() -> [[Bitboard; 64]; 64] {
    let mut rays = [[0; 64]; 64];

    let mut from = 0;
    while from < 64 {
        let mut to = 0;
        while to < 64 {
            rays[from][to] = get_in_between_ray(from as Square, to as Square);
            to += 1;
        }
        from += 1;
    }

    rays
}

const fn get_ray(mut from: Square, dir: Dir) -> Bitboard {
    let mut result = 0;
    from += dir.to_square();
    if dir.rem(8) == 1 {
        while (from + 8) % 8 > 0 && 0 <= from && from < 64 {
            result |= 1 << from;
            from += dir.to_square();
        }
    } else if dir.rem(8) == 7 {
        while (from + 8) % 8 < 7 && 0 <= from && from < 64 {
            result |= 1 << from;
            from += dir.to_square();
        }
    } else {
        while 0 <= from && from < 64 {
            result |= 1 << from;
            from += dir.to_square();
        }
    }
    result
}

const fn get_in_between_ray(mut from: Square, mut to: Square) -> Bitboard {
    if from > to {
        std::mem::swap(&mut from, &mut to);
    }
    let mut result = 0;
    if let Some(dir) = Dir::from_squares(from, to) {
        from += dir.to_square();
        if dir.rem(8) == 1 {
            while (from + 8) % 8 > 0 && from < to {
                result |= 1 << from;
                from += dir.to_square();
            }
        } else if dir.rem(8) == 7 {
            while (from + 8) % 8 < 7 && from < to {
                result |= 1 << from;
                from += dir.to_square();
            }
        } else {
            while 0 <= from && from < to {
                result |= 1 << from;
                from += dir.to_square();
            }
        }
    }
    result
}

fn get_positive_ray_attacks(square: Square, dir: Dir, occupied: Bitboard) -> Bitboard {
    let mut attacks = RAYS[dir as usize][square as usize];
    let blocker = attacks & occupied;
    if let Some(square) = blocker.bitscan_forward() {
        attacks ^= RAYS[dir as usize][square as usize];
    }
    attacks
}

fn get_negative_ray_attacks(square: Square, dir: Dir, occupied: Bitboard) -> Bitboard {
    let mut attacks = RAYS[dir as usize][square as usize];
    let blocker = attacks & occupied;
    if let Some(square) = blocker.bitscan_reverse() {
        attacks ^= RAYS[dir as usize][square as usize];
    }
    attacks
}

impl Board {
    pub fn generate_moves(&mut self) -> MoveGeneratorResult {
        MoveGenerator::generate_moves(self)
    }
}

struct MoveGenerator<'a> {
    board: &'a mut Board,

    attacks: Bitboard,
    checkers: Bitboard,
    block_ray: Bitboard,
    pinned: Bitboard,
}

#[derive(Debug)]
pub struct MoveGeneratorResult {
    pub moves: [Move; 218],
    pub in_check: bool,

    pub len: usize,
    index: usize,
}

impl MoveGeneratorResult {
    fn push(&mut self, m: Move) {
        self.moves[self.len] = m;
        self.len += 1;
    }

    fn append(&mut self, moves: &[Move]) {
        for m in moves {
            self.moves[self.len] = *m;
            self.len += 1;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn filter<P>(self, p: P) -> Self
    where
        P: Fn(&Move) -> bool,
    {
        let mut moves = MoveGeneratorResult {
            moves: [Move::NULL; 218],

            in_check: self.in_check,

            len: 0,
            index: 0,
        };

        for m in self {
            if p(&m) {
                moves.push(m);
            }
        }

        moves
    }

    pub fn sort_by<F>(&mut self, mut b: F)
    where
        F: FnMut(&Move, &Move) -> std::cmp::Ordering,
    {
        for i in 0..self.len {
            for j in i + 1..self.len {
                if b(&self.moves[i], &self.moves[j]) == std::cmp::Ordering::Greater {
                    let x = self.moves[i];
                    self.moves[i] = self.moves[j];
                    self.moves[j] = x;
                }
            }
        }
    }
}

impl Iterator for MoveGeneratorResult {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index <= self.len {
            Some(self.moves[self.index - 1])
        } else {
            None
        }
    }
}

impl<'a> MoveGenerator<'a> {
    fn generate_moves(board: &mut Board) -> MoveGeneratorResult {
        let mut move_generator = MoveGenerator {
            board: board,

            attacks: 0,
            checkers: 0,
            block_ray: 0,
            pinned: 0,
        };

        move_generator.get_attacks();
        move_generator.get_checkers();
        move_generator.get_block_ray();
        move_generator.get_pinned();

        let mut moves = MoveGeneratorResult {
            moves: [Move::NULL; 218],
            in_check: move_generator.checkers > 0,

            len: 0,
            index: 0,
        };

        if move_generator.checkers.count_ones() != 2 {
            move_generator.generate_pawn_moves(&mut moves);
            move_generator.generate_rook_moves(&mut moves);
            move_generator.generate_knight_moves(&mut moves);
            move_generator.generate_bishop_moves(&mut moves);
            move_generator.generate_queen_moves(&mut moves);
        }

        move_generator.generate_king_moves(&mut moves);

        moves
    }

    fn get_attacks(&mut self) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();
        let occupied = own | opponent;

        let mut pawns = self.board.pawns & opponent;
        while let Some(from) = pawns.pop_lsb() {
            self.attacks |= match self.board.turn {
                Color::White => self.black_pawn_attacks(from),
                Color::Black => self.white_pawn_attacks(from),
                Color::None => unreachable!(),
            };
        }
        let mut rooks = self.board.rooks & opponent;
        while let Some(from) = rooks.pop_lsb() {
            self.attacks |= self.rook_attacks(from, occupied);
        }
        let mut knights = self.board.knights & opponent;
        while let Some(from) = knights.pop_lsb() {
            self.attacks |= self.knight_attacks(from);
        }
        let mut bishops = self.board.bishops & opponent;
        while let Some(from) = bishops.pop_lsb() {
            self.attacks |= self.bishop_attacks(from, occupied);
        }
        let mut queens = self.board.queens & opponent;
        while let Some(from) = queens.pop_lsb() {
            self.attacks |= self.queen_attacks(from, occupied);
        }
        let mut kings = self.board.kings & opponent;
        while let Some(from) = kings.pop_lsb() {
            self.attacks |= self.king_attacks(from);
        }
    }

    fn get_checkers(&mut self) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();

        let occupied = own | opponent;

        let mut king = own & self.board.kings;
        let from = king.pop_lsb().expect("No king found");

        self.checkers = (match self.board.turn {
            Color::White => self.white_pawn_attacks(from) & self.board.pawns,
            Color::Black => self.black_pawn_attacks(from) & self.board.pawns,
            Color::None => unreachable!(),
        } | (self.rook_attacks(from, occupied) & self.board.rooks)
            | (self.knight_attacks(from) & self.board.knights)
            | (self.bishop_attacks(from, occupied) & self.board.bishops)
            | (self.queen_attacks(from, occupied) & self.board.queens)
            | (self.king_attacks(from) & self.board.kings))
            & opponent
    }

    fn get_block_ray(&mut self) {
        let own = self.board.own_pieces();
        self.block_ray = if let Some(checker) = self.checkers.bitscan_forward() {
            IN_BETWEEN_RAYS[checker as usize][(self.board.kings & own)
                .bitscan_forward()
                .expect("No king found") as usize]
                | self.checkers
        } else {
            Bitboard::MAX
        };
    }

    fn get_pinned(&mut self) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();
        let king = self.board.kings & own;
        let king_square = king.bitscan_forward().expect("No king found");

        let mut pinner = self.xray(
            |mg: &mut MoveGenerator<'_>, from: Square, occupied: Bitboard| {
                mg.rook_attacks(from, occupied)
            },
            king_square,
        ) & (self.board.rooks | self.board.queens)
            & opponent;

        while let Some(pinner_square) = pinner.pop_lsb() {
            self.pinned |= IN_BETWEEN_RAYS[pinner_square as usize][king_square as usize] & own;
        }

        pinner = self.xray(
            |mg: &mut MoveGenerator<'_>, from: Square, occupied: Bitboard| {
                mg.bishop_attacks(from, occupied)
            },
            king_square,
        ) & (self.board.bishops | self.board.queens)
            & opponent;

        while let Some(pinner_square) = pinner.pop_lsb() {
            self.pinned |= IN_BETWEEN_RAYS[pinner_square as usize][king_square as usize] & own;
        }
    }

    fn xray<T>(&mut self, attack_fn: T, from: Square) -> Bitboard
    where
        T: Fn(&mut MoveGenerator, Square, Bitboard) -> Bitboard,
    {
        let occupied = self.board.own_pieces() | self.board.opponent_pieces();
        let mut blockers = self.board.own_pieces();
        let attacks = attack_fn(self, from, occupied);
        blockers &= attacks;
        return attacks ^ attack_fn(self, from, occupied ^ blockers);
    }

    fn xray_dir(&mut self, from: Square, to: Square) -> Bitboard {
        let occupied = self.board.own_pieces() | self.board.opponent_pieces();
        let mut blockers = self.board.own_pieces();
        let dir = Dir::from_squares(from, to).unwrap();
        let attack_fn = if dir.to_square() > 0 {
            get_positive_ray_attacks
        } else {
            get_negative_ray_attacks
        };
        let attacks = attack_fn(from, dir, occupied);
        blockers &= attacks;
        return attack_fn(from, dir, occupied ^ blockers);
    }

    fn generate_pawn_moves(&mut self, moves: &mut MoveGeneratorResult) {
        match self.board.turn {
            Color::White => self.generate_white_pawn_moves(moves),
            Color::Black => self.generate_black_pawn_moves(moves),
            Color::None => unreachable!(),
        }
    }

    fn generate_white_pawn_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let mut pawns = self.board.pawns & self.board.white_pieces;
        let blockers = self.board.white_pieces | self.board.black_pieces;
        let free = !blockers;
        while let Some(from) = pawns.pop_lsb() {
            let mut bitboard = 0;
            if 1 << from + 8 & self.block_ray & free > 0 {
                bitboard |= 1 << from + 8;
            }
            if 1 << from + 7 & self.block_ray & self.board.black_pieces & NOT_H_FILE > 0 {
                bitboard |= 1 << from + 7;
            }
            if Bitboard::checked_shl(1, (from + 9) as u32).unwrap_or(0)
                & self.block_ray
                & self.board.black_pieces
                & NOT_A_FILE
                > 0
            {
                bitboard |= 1 << from + 9;
            }
            if from / 8 == 1
                && (1 << from + 8 | 1 << from + 16) & blockers == 0
                && self.block_ray & 1 << from + 16 > 0
            {
                bitboard |= 1 << from + 16;
            }
            if self.board.ep != -1 {
                self.white_pawn_en_passant(&mut bitboard, from);
            }

            if 1 << from & self.pinned > 0 {
                bitboard &= self.xray_dir(
                    (self.board.kings & self.board.white_pieces)
                        .bitscan_forward()
                        .expect("No king found"),
                    from,
                );
            }

            while let Some(to) = bitboard.pop_lsb() {
                let flags = if 1 << to & self.board.black_pieces > 0 {
                    0b0100
                } else {
                    0
                } | if to - from == 16 { 0b0001 } else { 0 }
                    | if to == self.board.ep { 0b0101 } else { 0 };
                moves.append(&mut Move::add_promotion_if_possible(from, to, flags));
            }
        }
    }

    fn white_pawn_en_passant(&mut self, bitboard: &mut Bitboard, from: Square) {
        let king_square = (self.board.kings & self.board.white_pieces)
            .pop_lsb()
            .expect("No king found");
        if 1 << from + 7 & (self.block_ray | self.checkers << 8) & 1 << self.board.ep & NOT_H_FILE
            > 0
        {
            if king_square / 8 == 4 {
                let dir = Dir::from_squares(king_square, from).unwrap();
                if dir.to_square() > 0
                    && get_positive_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from - 1,
                    ) & self.board.black_pieces
                        & (self.board.rooks | self.board.queens)
                        == 0
                {
                    *bitboard |= 1 << from + 7;
                } else if dir.to_square() < 0
                    && get_negative_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from - 1,
                    ) & self.board.black_pieces
                        & (self.board.rooks | self.board.queens)
                        == 0
                {
                    *bitboard |= 1 << from + 7;
                }
            } else {
                *bitboard |= 1 << from + 7;
            }
        }
        if Bitboard::checked_shl(1, (from + 9) as u32).unwrap_or(0)
            & (self.block_ray | self.checkers << 8)
            & 1 << self.board.ep
            & NOT_A_FILE
            > 0
        {
            if king_square / 8 == 4 {
                let dir = Dir::from_squares(king_square, from).unwrap();
                if dir.to_square() > 0
                    && get_positive_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from,
                    ) & self.board.black_pieces
                        & (self.board.queens | self.board.rooks)
                        == 0
                {
                    *bitboard |= 1 << from + 9;
                } else if dir.to_square() < 0
                    && get_negative_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from,
                    ) & self.board.black_pieces
                        & (self.board.queens | self.board.rooks)
                        == 0
                {
                    *bitboard |= 1 << from + 9;
                }
            } else {
                *bitboard |= 1 << from + 9;
            }
        }
    }

    fn white_pawn_attacks(&self, from: Square) -> Bitboard {
        (Bitboard::checked_shl(1, (from + 7) as u32).unwrap_or(0) & NOT_H_FILE)
            | (Bitboard::checked_shl(1, (from + 9) as u32).unwrap_or(0) & NOT_A_FILE)
    }

    fn generate_black_pawn_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let mut pawns = self.board.pawns & self.board.black_pieces;
        let blockers = self.board.white_pieces | self.board.black_pieces;
        let free = !blockers;
        while let Some(from) = pawns.pop_lsb() {
            let mut bitboard = 0;
            if 1 << from - 8 & self.block_ray & free > 0 {
                bitboard |= 1 << from - 8;
            }
            if 1 << from - 7 & self.block_ray & self.board.white_pieces & NOT_A_FILE > 0 {
                bitboard |= 1 << from - 7;
            }
            if Bitboard::checked_shl(1, (from - 9) as u32).unwrap_or(0)
                & self.block_ray
                & self.board.white_pieces
                & NOT_H_FILE
                > 0
            {
                bitboard |= 1 << from - 9;
            }
            if from / 8 == 6
                && (1 << from - 8 | 1 << from - 16) & blockers == 0
                && self.block_ray & 1 << from - 16 > 0
            {
                bitboard |= 1 << from - 16;
            }
            if self.board.ep != -1 {
                self.black_pawn_en_passant(&mut bitboard, from);
            }

            if 1 << from & self.pinned > 0 {
                bitboard &= self.xray_dir(
                    (self.board.kings & self.board.black_pieces)
                        .bitscan_forward()
                        .expect("No king found"),
                    from,
                );
            }

            while let Some(to) = bitboard.pop_lsb() {
                let flags = if 1 << to & self.board.white_pieces > 0 {
                    0b0100
                } else {
                    0
                } | if to - from == -16 { 0b0001 } else { 0 }
                    | if to == self.board.ep { 0b0101 } else { 0 };
                moves.append(&mut Move::add_promotion_if_possible(from, to, flags));
            }
        }
    }

    fn black_pawn_en_passant(&mut self, bitboard: &mut u64, from: i16) {
        let king_square = (self.board.kings & self.board.black_pieces)
            .pop_lsb()
            .expect("No king found");
        if 1 << from - 7 & (self.block_ray | self.checkers >> 8) & 1 << self.board.ep & NOT_A_FILE
            > 0
        {
            if king_square / 8 == 3 {
                let dir = Dir::from_squares(king_square, from).unwrap();
                if dir.to_square() > 0
                    && get_positive_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from,
                    ) & self.board.white_pieces
                        & (self.board.rooks | self.board.queens)
                        == 0
                {
                    *bitboard |= 1 << from - 7;
                } else if dir.to_square() < 0
                    && get_negative_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from,
                    ) & self.board.white_pieces
                        & (self.board.rooks | self.board.queens)
                        == 0
                {
                    *bitboard |= 1 << from - 7;
                }
            } else {
                *bitboard |= 1 << from - 7;
            }
        }
        if Bitboard::checked_shl(1, (from - 9) as u32).unwrap_or(0)
            & (self.block_ray | self.checkers >> 8)
            & 1 << self.board.ep
            & NOT_H_FILE
            > 0
        {
            if king_square / 8 == 3 {
                let dir = Dir::from_squares(king_square, from).unwrap();
                if dir.to_square() > 0
                    && get_positive_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from - 1,
                    ) & self.board.white_pieces
                        & (self.board.rooks | self.board.queens)
                        == 0
                {
                    *bitboard |= 1 << from - 9;
                } else if dir.to_square() < 0
                    && get_negative_ray_attacks(
                        king_square,
                        dir,
                        (self.board.white_pieces | self.board.black_pieces) ^ 0b11 << from - 1,
                    ) & self.board.white_pieces
                        & (self.board.rooks | self.board.queens)
                        == 0
                {
                    *bitboard |= 1 << from - 9;
                }
            } else {
                *bitboard |= 1 << from - 9;
            }
        }
    }

    fn black_pawn_attacks(&self, from: Square) -> Bitboard {
        (Bitboard::checked_shl(1, (from - 7) as u32).unwrap_or(0) & NOT_A_FILE)
            | (Bitboard::checked_shl(1, (from - 9) as u32).unwrap_or(0) & NOT_H_FILE)
    }

    fn generate_rook_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut rooks = self.board.rooks & own;
        while let Some(from) = rooks.pop_lsb() {
            let mut bitboard = self.rook_attacks(from, occupied) & self.block_ray & free;

            if 1 << from & self.pinned > 0 {
                bitboard &= self.xray_dir(
                    (self.board.kings & own)
                        .bitscan_forward()
                        .expect("No king found"),
                    from,
                );
            }

            while let Some(to) = bitboard.pop_lsb() {
                let flags = if 1 << to & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }
    }

    fn rook_attacks(&self, from: Square, occupied: Bitboard) -> Bitboard {
        get_positive_ray_attacks(from, Dir::North, occupied)
            | get_positive_ray_attacks(from, Dir::East, occupied)
            | get_negative_ray_attacks(from, Dir::West, occupied)
            | get_negative_ray_attacks(from, Dir::South, occupied)
    }

    fn generate_knight_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();

        let free = !own;

        let mut knights = self.board.knights & own & !self.pinned;

        while let Some(from) = knights.pop_lsb() {
            let mut bitboard = self.knight_attacks(from) & self.block_ray & free;

            while let Some(to) = bitboard.pop_lsb() {
                let flags = if 1 << to & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }
    }

    fn knight_attacks(&self, from: Square) -> Bitboard {
        (Bitboard::checked_shl(1, (from + 15) as u32).unwrap_or(0) & NOT_H_FILE)
            | (Bitboard::checked_shl(1, (from + 17) as u32).unwrap_or(0) & NOT_A_FILE)
            | (Bitboard::checked_shl(1, (from + 6) as u32).unwrap_or(0) & NOT_GH_FILE)
            | (Bitboard::checked_shl(1, (from + 10) as u32).unwrap_or(0) & NOT_AB_FILE)
            | (Bitboard::checked_shl(1, (from - 10) as u32).unwrap_or(0) & NOT_GH_FILE)
            | (Bitboard::checked_shl(1, (from - 6) as u32).unwrap_or(0) & NOT_AB_FILE)
            | (Bitboard::checked_shl(1, (from - 17) as u32).unwrap_or(0) & NOT_H_FILE)
            | (Bitboard::checked_shl(1, (from - 15) as u32).unwrap_or(0) & NOT_A_FILE)
    }

    fn generate_bishop_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut bishops = self.board.bishops & own;
        while let Some(from) = bishops.pop_lsb() {
            let mut bitboard = self.bishop_attacks(from, occupied) & self.block_ray & free;
            if 1 << from & self.pinned > 0 {
                bitboard &= self.xray_dir(
                    (self.board.kings & own)
                        .bitscan_forward()
                        .expect("No king found"),
                    from,
                );
            }

            while let Some(to) = bitboard.pop_lsb() {
                let flags = if 1 << to & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }
    }

    fn bishop_attacks(&self, from: Square, occupied: Bitboard) -> Bitboard {
        get_positive_ray_attacks(from, Dir::NorthWest, occupied)
            | get_positive_ray_attacks(from, Dir::NorthEast, occupied)
            | get_negative_ray_attacks(from, Dir::SouthEast, occupied)
            | get_negative_ray_attacks(from, Dir::SouthWest, occupied)
    }

    fn generate_queen_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut queens = self.board.queens & own;
        while let Some(from) = queens.pop_lsb() {
            let mut bitboard = self.queen_attacks(from, occupied) & self.block_ray & free;

            if 1 << from & self.pinned > 0 {
                bitboard &= self.xray_dir(
                    (self.board.kings & own)
                        .bitscan_forward()
                        .expect("No king found"),
                    from,
                );
            }

            while let Some(to) = bitboard.pop_lsb() {
                let flags = if 1 << to & opponent > 0 { 0b0100 } else { 0 };
                moves.push(Move::new(from, to, flags));
            }
        }
    }

    fn queen_attacks(&self, from: Square, occupied: Bitboard) -> Bitboard {
        get_positive_ray_attacks(from, Dir::NorthWest, occupied)
            | get_positive_ray_attacks(from, Dir::North, occupied)
            | get_positive_ray_attacks(from, Dir::NorthEast, occupied)
            | get_positive_ray_attacks(from, Dir::East, occupied)
            | get_negative_ray_attacks(from, Dir::SouthEast, occupied)
            | get_negative_ray_attacks(from, Dir::South, occupied)
            | get_negative_ray_attacks(from, Dir::SouthWest, occupied)
            | get_negative_ray_attacks(from, Dir::West, occupied)
    }

    fn generate_king_moves(&mut self, moves: &mut MoveGeneratorResult) {
        let own = self.board.own_pieces();
        let opponent = self.board.opponent_pieces();

        let occupied = own | opponent;

        let free = !own;

        let mut king = self.board.kings & own;
        let from = king.pop_lsb().expect("No king found");

        let mut bitboard = self.king_attacks(from) & free & !self.attacks;
        while let Some(checker_square) = self.checkers.pop_lsb() {
            if 1 << checker_square & self.board.pawns == 0
                && let Some(dir) = Dir::from_squares(checker_square, from)
            {
                bitboard &= !(1 << from + dir.to_square());
            }
        }

        while let Some(to) = bitboard.pop_lsb() {
            let flags = if 1 << to & opponent > 0 { 0b0100 } else { 0 };
            moves.push(Move::new(from, to, flags));
        }

        // Castling
        match self.board.turn {
            Color::White => {
                if self.board.castling_rights & 0b1000 > 0
                    && occupied & 0b01100000 == 0
                    && !self.attacks & 0b01110000 == 0b01110000
                {
                    moves.push(Move::new(from, from + 2, 0b0010));
                }
                if self.board.castling_rights & 0b0100 > 0
                    && occupied & 0b00001110 == 0
                    && !self.attacks & 0b00011100 == 0b00011100
                {
                    moves.push(Move::new(from, from - 2, 0b0011));
                }
            }
            Color::Black => {
                if self.board.castling_rights & 0b0010 > 0
                    && occupied & (0b01100000 << 56) == 0
                    && !self.attacks & (0b01110000 << 56) == 0b01110000 << 56
                {
                    moves.push(Move::new(from, from + 2, 0b0010));
                }
                if self.board.castling_rights & 0b0001 > 0
                    && occupied & (0b00001110 << 56) == 0
                    && !self.attacks & (0b00011100 << 56) == 0b00011100 << 56
                {
                    moves.push(Move::new(from, from - 2, 0b0011));
                }
            }
            Color::None => unreachable!(),
        }
    }

    fn king_attacks(&self, from: Square) -> Bitboard {
        let mut bitboard = (Bitboard::checked_shl(1, (from - 1) as u32).unwrap_or(0) & NOT_H_FILE)
            | Bitboard::checked_shl(1, from as u32).unwrap_or(0)
            | (Bitboard::checked_shl(1, (from + 1) as u32).unwrap_or(0) & NOT_A_FILE);
        bitboard |= bitboard.checked_shl(8_u32).unwrap_or(0);
        bitboard |= bitboard.checked_shr(8_u32).unwrap_or(0);
        bitboard
    }
}
