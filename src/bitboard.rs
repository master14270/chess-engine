use crate::constants;

pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn idx(&self) -> usize {
        match self {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

pub const NOT_FILE_A: u64 = 18374403900871474942;
pub const NOT_FILE_B: u64 = 18302063728033398269;
pub const NOT_FILE_AB: u64 = 18229723555195321596;
pub const NOT_FILE_G: u64 = 13816973012072644543;
pub const NOT_FILE_H: u64 = 9187201950435737471;
pub const NOT_FILE_GH: u64 = 4557430888798830399;

pub const NOT_RANK_8: u64 = 18446744073709551360;
pub const NOT_RANK_7: u64 = 18446744073709486335;
pub const NOT_RANK_2: u64 = 18374967954648334335;
pub const NOT_RANK_1: u64 = 72057594037927935;

/*
    All Squares: 18446744073709551615

*/

pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    pub fn to_char(&self, side: Color) -> char {
        let c = match self {
            Self::King => 'k',
            Self::Queen => 'q',
            Self::Rook => 'r',
            Self::Bishop => 'b',
            Self::Knight => 'n',
            Self::Pawn => 'p',
        };

        return match side {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        };
    }
}

pub struct Constants {
    pub pawn_attacks: [[u64; 64]; 2],
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],
    pub bishop_attacks: Vec<Vec<u64>>, // [64][512]
    pub rook_attacks: Vec<Vec<u64>>,   // [64][4096]
}

impl Constants {
    pub fn new() -> Self {
        let mut pawn_attacks: [[u64; 64]; 2] = [[0; 64]; 2];
        let mut knight_attacks: [u64; 64] = [0; 64];
        let mut king_attacks: [u64; 64] = [0; 64];

        // These are too big to put on the stack.
        let mut bishop_attacks: Vec<Vec<u64>> = vec![vec![0; 512]; 64];
        let mut rook_attacks: Vec<Vec<u64>> = vec![vec![0; 4096]; 64];

        for square in 0..64 {
            pawn_attacks[Color::White.idx()][square] =
                mask_pawn_attacks(square as u64, Color::White);
            pawn_attacks[Color::Black.idx()][square] =
                mask_pawn_attacks(square as u64, Color::Black);

            knight_attacks[square] = mask_knight_attacks(square as u64);
            king_attacks[square] = mask_king_attacks(square as u64);
        }

        init_slider_attacks(true, &mut bishop_attacks, &mut rook_attacks);
        init_slider_attacks(false, &mut bishop_attacks, &mut rook_attacks);

        return Constants {
            pawn_attacks,
            knight_attacks,
            king_attacks,
            bishop_attacks,
            rook_attacks,
        };
    }
}

// TODO: Research more on lifetime stuff.
pub struct ChessGame<'a> {
    pub bitboard_constants: &'a Constants,

    // En-Passant
    pub en_passant_target: Option<usize>,

    // Flags.
    pub white_to_move: bool,
    pub can_white_castle_long: bool,
    pub can_white_castle_short: bool,
    pub can_black_castle_long: bool,
    pub can_black_castle_short: bool,

    // White Pieces (bitboards)
    pub white_pawns: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_knights: u64,
    pub white_queens: u64,
    pub white_king: u64,

    // Black Pieces (bitboards)
    pub black_pawns: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_knights: u64,
    pub black_queens: u64,
    pub black_king: u64,

    // Occupancies (bitboards)
    pub white_occupancies: u64,
    pub black_occupancies: u64,
    pub all_occupancies: u64,
}

impl<'a> ChessGame<'a> {
    pub fn new(c: &'a Constants) -> Self {
        return ChessGame {
            bitboard_constants: c,

            en_passant_target: None,

            white_to_move: true,
            can_white_castle_long: true,
            can_white_castle_short: true,
            can_black_castle_long: true,
            can_black_castle_short: true,

            white_pawns: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_knights: 0,
            white_queens: 0,
            white_king: 0,

            black_pawns: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_knights: 0,
            black_queens: 0,
            black_king: 0,

            white_occupancies: 0,
            black_occupancies: 0,
            all_occupancies: 0,
        };
    }

    // Takes all pieces off the board.
    pub fn clear_board(&mut self) {
        self.white_pawns = 0;
        self.white_bishops = 0;
        self.white_rooks = 0;
        self.white_knights = 0;
        self.white_queens = 0;
        self.white_king = 0;

        self.black_pawns = 0;
        self.black_bishops = 0;
        self.black_rooks = 0;
        self.black_knights = 0;
        self.black_queens = 0;
        self.black_king = 0;

        self.white_occupancies = 0;
        self.black_occupancies = 0;
        self.all_occupancies = 0;
    }

    // TODO: Finish this...
    pub fn print_board(&self) {
        // do something...
        println!("    A   B   C   D   E   F   G   H");
        println!("  |---|---|---|---|---|---|---|---|");
        for rank in 0..8 {
            print!("{} |", 8 - rank);
            for file in 0..8 {
                let square: u64 = rank * 8 + file;

                let (piece_type, color) = self.get_piece_at_square(square);

                // If nothing is there, print a space and go next.
                if piece_type.is_none() {
                    print!("   |");
                    continue;
                }

                let c = piece_type.unwrap().to_char(color.unwrap());
                print!(" {} |", c);
            }
            print!(" {}", 8 - rank);
            println!();
            println!("  |---|---|---|---|---|---|---|---|");
        }
        println!("    A   B   C   D   E   F   G   H");
    }

    // TODO: Consider robust error handling rather than `panic` usage.
    pub fn import_fen(&mut self, fen: &str) -> Result<(), String> {
        // Clear the board.
        self.clear_board();

        // Trim the string.
        let trimmed_full_fen = fen.trim();

        // Split by first space to separate board position from the rest of the FEN components.
        let mut parts = trimmed_full_fen.split(' ');
        let board_str = match parts.next() {
            Some(s) => s,
            None => return Err("No board position found in FEN.".to_string()),
        };

        // Prepare to populate our board.
        let rows = board_str.split('/');
        let mut y_pos = 0;

        // Parse each row of the board.
        for row in rows {
            // For each char in the row, we will either have a character or a number.
            let mut x_pos = 0;
            for c in row.chars() {
                // Handles empty spaces on the board.
                if c.is_digit(10) {
                    let num_empties = match c.to_digit(10) {
                        Some(n) => n,
                        None => return Err(format!("Failed to parse digit: {}.", c)),
                    };

                    if num_empties < 1 || num_empties > 8 {
                        return Err(format!("Invalid number of empty spaces: {}", num_empties));
                    }

                    x_pos += num_empties;
                    continue;
                }

                // Place the piece on the board.
                let piece_color: Color;
                if c.is_ascii_uppercase() {
                    piece_color = Color::White;
                } else {
                    piece_color = Color::Black;
                }

                let piece_type = match c.to_ascii_lowercase() {
                    'k' => PieceType::King,
                    'q' => PieceType::Queen,
                    'r' => PieceType::Rook,
                    'b' => PieceType::Bishop,
                    'n' => PieceType::Knight,
                    'p' => PieceType::Pawn,
                    _ => return Err(format!("Unexpected piece letter {}", c)),
                };

                let square = y_pos * 8 + x_pos;
                self.place_piece_on_board(piece_color, piece_type, square as u64);

                x_pos += 1;
            }

            // Ensure that the board has exactly 8 cols.
            if x_pos != 8 {
                return Err(format!(
                    "Board must have exactly 8 columns. We parsed: {}",
                    x_pos
                ));
            }

            y_pos += 1;
        }

        // Ensure that the board has exactly 8 rows.
        if y_pos != 8 {
            return Err(format!(
                "Board must have exactly 8 rows. We parsed: {}",
                y_pos
            ));
        }

        // Store whose turn it is to move.
        let whose_turn = match parts.next() {
            Some(s) => s,
            None => return Err("Unsure whose turn it is. Cannot proceed.".to_string()),
        };

        if whose_turn.to_ascii_lowercase() == "w" {
            self.white_to_move = true;
        } else if whose_turn.to_ascii_lowercase() == "b" {
            self.white_to_move = false;
        } else {
            return Err(format!(
                "Unexpected character for whose turn it is: {}. Should be 'w' or 'b'.",
                whose_turn
            ));
        }

        // Castling.
        let castling_rights_str = parts.next();
        match castling_rights_str {
            Some(s) => {
                // Assume no one can castle.
                self.can_white_castle_long = false;
                self.can_white_castle_short = false;
                self.can_black_castle_long = false;
                self.can_black_castle_short = false;

                // Update rights based on what we find in the string.
                for c in s.chars() {
                    match c {
                        'K' => self.can_white_castle_short = true,
                        'Q' => self.can_white_castle_long = true,
                        'k' => self.can_black_castle_short = true,
                        'q' => self.can_black_castle_long = true,
                        _ => (),
                    }
                }
            }
            None => return Ok(()),
        }

        // En-Passant target.
        let en_passant_target_str = parts.next();
        match en_passant_target_str {
            Some(s) => {
                // Try to parse the string as a coordinate.
                let parsed_coord = str_coord_to_square(s);
                if parsed_coord.is_ok() {
                    self.en_passant_target = Some(parsed_coord.unwrap());
                } else {
                    self.en_passant_target = None;
                }
            }
            None => return Ok(()),
        };

        // Half-Moves since last pawn move and capture (for 50 move rule).
        // let half_move_str = parts.next();
        // match half_move_str {
        //     Some(s) => {
        //         let parsed_number: Result<u32, _> = s.parse();
        //         if parsed_number.is_ok() {
        //             self.half_move_count_non_pawn_non_capture = parsed_number.unwrap();
        //         }
        //     }
        //     None => return,
        // }

        // Full move count. Incremented after black moves.
        // let full_move_count_str = parts.next();
        // match full_move_count_str {
        //     Some(s) => {
        //         let parsed_number: Result<u32, _> = s.parse();
        //         if parsed_number.is_ok() {
        //             self.full_move_count = parsed_number.unwrap();
        //         }
        //     }
        //     None => return,
        // }

        return Ok(());
    }

    pub fn place_piece_on_board(&mut self, side: Color, piece_type: PieceType, square: u64) {
        match side {
            Color::White => {
                match piece_type {
                    PieceType::Pawn => self.white_pawns = set_bit(self.white_pawns, square),
                    PieceType::Bishop => self.white_bishops = set_bit(self.white_bishops, square),
                    PieceType::Knight => self.white_knights = set_bit(self.white_knights, square),
                    PieceType::Rook => self.white_rooks = set_bit(self.white_rooks, square),
                    PieceType::Queen => self.white_queens = set_bit(self.white_queens, square),
                    PieceType::King => self.white_king = set_bit(self.white_king, square),
                };
                self.white_occupancies = set_bit(self.white_occupancies, square);
            }
            Color::Black => {
                match piece_type {
                    PieceType::Pawn => self.black_pawns = set_bit(self.black_pawns, square),
                    PieceType::Bishop => self.black_bishops = set_bit(self.black_bishops, square),
                    PieceType::Knight => self.black_knights = set_bit(self.black_knights, square),
                    PieceType::Rook => self.black_rooks = set_bit(self.black_rooks, square),
                    PieceType::Queen => self.black_queens = set_bit(self.black_queens, square),
                    PieceType::King => self.black_king = set_bit(self.black_king, square),
                };
                self.black_occupancies = set_bit(self.black_occupancies, square);
            }
        };

        self.all_occupancies = set_bit(self.all_occupancies, square);
    }

    // WARNING: Not efficient function??
    pub fn get_piece_at_square(&self, square: u64) -> (Option<PieceType>, Option<Color>) {
        let is_occupied = get_bit(self.all_occupancies, square) != 0;
        if !is_occupied {
            return (None, None);
        }

        let is_occupied_white = get_bit(self.white_occupancies, square) != 0;
        let is_occupied_black = get_bit(self.black_occupancies, square) != 0;

        if is_occupied_white {
            if get_bit(self.white_pawns, square) != 0 {
                return (Some(PieceType::Pawn), Some(Color::White));
            } else if get_bit(self.white_bishops, square) != 0 {
                return (Some(PieceType::Bishop), Some(Color::White));
            } else if get_bit(self.white_knights, square) != 0 {
                return (Some(PieceType::Knight), Some(Color::White));
            } else if get_bit(self.white_rooks, square) != 0 {
                return (Some(PieceType::Rook), Some(Color::White));
            } else if get_bit(self.white_queens, square) != 0 {
                return (Some(PieceType::Queen), Some(Color::White));
            } else if get_bit(self.white_king, square) != 0 {
                return (Some(PieceType::King), Some(Color::White));
            } else {
                panic!("Something has gone very wrong.");
            }
        } else if is_occupied_black {
            if get_bit(self.black_pawns, square) != 0 {
                return (Some(PieceType::Pawn), Some(Color::Black));
            } else if get_bit(self.black_bishops, square) != 0 {
                return (Some(PieceType::Bishop), Some(Color::Black));
            } else if get_bit(self.black_knights, square) != 0 {
                return (Some(PieceType::Knight), Some(Color::Black));
            } else if get_bit(self.black_rooks, square) != 0 {
                return (Some(PieceType::Rook), Some(Color::Black));
            } else if get_bit(self.black_queens, square) != 0 {
                return (Some(PieceType::Queen), Some(Color::Black));
            } else if get_bit(self.black_king, square) != 0 {
                return (Some(PieceType::King), Some(Color::Black));
            } else {
                panic!("Something has gone very wrong.");
            }
        } else {
            panic!("Someting has gone very wrong.");
        }
    }

    // Bitwise operations make this pretty quick.
    pub fn is_square_attacked(&self, square: usize, who_is_attacking: Color) -> bool {
        match who_is_attacking {
            Color::White => {
                // Pawns.
                if self.bitboard_constants.pawn_attacks[Color::Black.idx()][square]
                    & self.white_pawns
                    != 0
                {
                    return true;
                }

                // Knights.
                if self.bitboard_constants.knight_attacks[square] & self.white_knights != 0 {
                    return true;
                }

                // Bishops.
                if self.get_bishop_attacks(square, self.all_occupancies) & self.white_bishops != 0 {
                    return true;
                }

                // Rooks.
                if self.get_rook_attacks(square, self.all_occupancies) & self.white_rooks != 0 {
                    return true;
                }

                // Queens. (we could speed this up slightly... look here for optimization if needed.)
                if self.get_queen_attacks(square, self.all_occupancies) & self.white_queens != 0 {
                    return true;
                }

                // King.
                if self.bitboard_constants.king_attacks[square] & self.white_king != 0 {
                    return true;
                }
            }
            Color::Black => {
                // Pawns.
                if self.bitboard_constants.pawn_attacks[Color::White.idx()][square]
                    & self.black_pawns
                    != 0
                {
                    return true;
                }

                // Knights.
                if self.bitboard_constants.knight_attacks[square] & self.black_knights != 0 {
                    return true;
                }

                // Bishops.
                if self.get_bishop_attacks(square, self.all_occupancies) & self.black_bishops != 0 {
                    return true;
                }

                // Rooks.
                if self.get_rook_attacks(square, self.all_occupancies) & self.black_rooks != 0 {
                    return true;
                }

                // Queens. (we could speed this up slightly... look here for optimization if needed.)
                if self.get_queen_attacks(square, self.all_occupancies) & self.black_queens != 0 {
                    return true;
                }

                // King.
                if self.bitboard_constants.king_attacks[square] & self.black_king != 0 {
                    return true;
                }
            }
        };

        // No attacks found!
        return false;
    }

    pub fn get_bishop_attacks(&self, square: usize, mut occupancy: u64) -> u64 {
        occupancy &= constants::BISHOP_MASKED_ATTACKS[square];
        (occupancy, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
        occupancy >>= 64 - constants::BISHOP_RELEVANT_BITS[square];

        return self.bitboard_constants.bishop_attacks[square][occupancy as usize];
    }

    pub fn get_rook_attacks(&self, square: usize, mut occupancy: u64) -> u64 {
        occupancy &= constants::ROOK_MASKED_ATTACKS[square];
        (occupancy, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
        occupancy >>= 64 - constants::ROOK_RELEVANT_BITS[square];

        return self.bitboard_constants.rook_attacks[square][occupancy as usize];
    }

    pub fn get_queen_attacks(&self, square: usize, occupancy: u64) -> u64 {
        return self.get_bishop_attacks(square, occupancy)
            | self.get_rook_attacks(square, occupancy);
    }
}

pub fn print_bitboard(bitboard: u64) {
    println!("    A   B   C   D   E   F   G   H");
    println!("  |---|---|---|---|---|---|---|---|");
    for rank in 0..8 {
        print!("{} |", 8 - rank);
        for file in 0..8 {
            let square: u64 = rank * 8 + file;
            let calc = get_bit(bitboard, square);
            let populated;
            if calc != 0 {
                populated = 1;
            } else {
                populated = 0;
            }

            print!(" {} |", populated);
        }
        print!(" {}", 8 - rank);
        println!();
        println!("  |---|---|---|---|---|---|---|---|");
    }
    println!("    A   B   C   D   E   F   G   H");
    println!("Bitboard Value: {bitboard}");
}

pub fn mask_pawn_attacks(square: u64, side: Color) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    match side {
        Color::White => {
            // Attacking top right.
            attacks |= (bitboard >> 7) & NOT_FILE_A;

            // Attacking top left.
            attacks |= (bitboard >> 9) & NOT_FILE_H;
        }
        Color::Black => {
            // Attacking bottom right.
            attacks |= (bitboard << 9) & NOT_FILE_A;

            // Attacking bottom left.
            attacks |= (bitboard << 7) & NOT_FILE_H;
        }
    }

    return attacks;
}

pub fn mask_knight_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    attacks |= (bitboard >> 10) & NOT_FILE_GH; // Up 1 Left 2
    attacks |= (bitboard >> 17) & NOT_FILE_H; // Up 2 Left 1

    attacks |= (bitboard >> 6) & NOT_FILE_AB; // Up 1 Right 2
    attacks |= (bitboard >> 15) & NOT_FILE_A; // Up 2 Right 1

    attacks |= (bitboard << 6) & NOT_FILE_GH; // Down 1 Left 2
    attacks |= (bitboard << 15) & NOT_FILE_H; // Down 2 Left 1

    attacks |= (bitboard << 10) & NOT_FILE_AB; // Down 1 Right 2
    attacks |= (bitboard << 17) & NOT_FILE_A; // Down 2 Right 1

    return attacks;
}

pub fn mask_king_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    // Put the piece on the board.
    bitboard = set_bit(bitboard, square);

    attacks |= (bitboard >> 9) & NOT_FILE_H; // Up 1 Left 1
    attacks |= bitboard >> 8; // Up 1
    attacks |= (bitboard >> 7) & NOT_FILE_A; // Up 1 Right 1

    attacks |= (bitboard >> 1) & NOT_FILE_H; // Left 1
    attacks |= (bitboard << 1) & NOT_FILE_A; // Right 1

    attacks |= (bitboard << 7) & NOT_FILE_H; // Down 1 Left 1
    attacks |= bitboard << 8; // Down 1
    attacks |= (bitboard << 9) & NOT_FILE_A; // Down 1 Right 1

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn mask_bishop_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Bottom Right Diagonal.
    rank = target_rank + 1;
    file = target_file + 1;
    while rank <= 6 && file <= 6 {
        attacks |= 1 << (rank * 8 + file);

        rank += 1;
        file += 1;
    }

    // Bottom Left Diagonal. Rust does not like subtraction overflow :D
    rank = target_rank + 1;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank <= 6 && file >= 1 {
        attacks |= 1 << (rank * 8 + file);

        rank += 1;
        file -= 1;
    }

    // Top Right Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file + 1;
    while rank >= 1 && file <= 6 {
        attacks |= 1 << (rank * 8 + file);

        rank -= 1;
        file += 1;
    }

    // Top Left Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank >= 1 && file >= 1 {
        attacks |= 1 << (rank * 8 + file);

        rank -= 1;
        file -= 1;
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn dynamic_bishop_attacks(square: u64, block: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Bottom Right Diagonal.
    rank = target_rank + 1;
    file = target_file + 1;
    while rank <= 7 && file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank += 1;
        file += 1;
    }

    // Bottom Left Diagonal. Rust does not like subtraction overflow :D
    rank = target_rank + 1;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while rank <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        // Break the loop when we would subtract underflow.
        rank += 1;
        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    // Top Right Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file + 1;
    while file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
        file += 1;
    }

    // Top Left Diagonal.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn mask_rook_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Right.
    rank = target_rank;
    file = target_file + 1;
    while file <= 6 {
        attacks |= 1 << (rank * 8 + file);
        file += 1;
    }

    // Left.
    rank = target_rank;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    while file >= 1 {
        attacks |= 1 << (rank * 8 + file);
        file -= 1;
    }

    // Up.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file;
    while rank >= 1 {
        attacks |= 1 << (rank * 8 + file);
        rank -= 1;
    }

    // Down.
    rank = target_rank + 1;
    file = target_file;
    while rank <= 6 {
        attacks |= 1 << (rank * 8 + file);
        rank += 1;
    }

    return attacks;
}

// Function a bit different than the others, it doesn't actually generate all the attacks...
pub fn dynamic_rook_attacks(square: u64, block: u64) -> u64 {
    let mut attacks: u64 = 0;

    let target_rank: u64 = square / 8;
    let target_file: u64 = square % 8;

    let mut rank: u64;
    let mut file: u64;

    // Right.
    rank = target_rank;
    file = target_file + 1;
    while file <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        file += 1;
    }

    // Left.
    rank = target_rank;
    file = match target_file.checked_sub(1) {
        Some(f) => f,
        None => 0,
    };
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        file = match file.checked_sub(1) {
            Some(f) => f,
            None => break,
        };
    }

    // Up.
    rank = match target_rank.checked_sub(1) {
        Some(r) => r,
        None => 0,
    };
    file = target_file;
    loop {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank = match rank.checked_sub(1) {
            Some(r) => r,
            None => break,
        };
    }

    // Down.
    rank = target_rank + 1;
    file = target_file;
    while rank <= 7 {
        let focus_square: u64 = 1 << (rank * 8 + file);
        attacks |= focus_square;

        // Exit if we are blocked.
        if (focus_square & block) != 0 {
            break;
        }

        rank += 1;
    }

    return attacks;
}

// Should this be an enum?
pub fn str_coord_to_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!(
            "Invalid input detected. Expected 2 chars. Got: `{}`.",
            s.len()
        ));
    }

    // We know the length is 2, so we can safely unwrap here.
    let file_str = s.chars().nth(0).unwrap().to_ascii_lowercase();
    let rank_str = s.chars().nth(1).unwrap();

    // Attempt conversion for file letter.
    let file: usize = file_str as usize - 'a' as usize;
    if file >= 8 as usize {
        return Err(format!("Invalid file letter: {}", file_str));
    }

    let rank: usize = match rank_str.to_digit(10) {
        Some(n) => (8 - n).try_into().unwrap(),
        None => return Err(format!("Unable to convert `{}` to a digit.", rank_str)),
    };

    return Ok(rank * 8 + file);
}

// I don't understand how this works yet...
pub fn set_occupancies(index: usize, bits_in_mask: usize, mut attack_mask: u64) -> u64 {
    let mut occupancy: u64 = 0;

    // Loop over bit range in attack mask.
    for count in 0..bits_in_mask {
        // Get LSB of attack mask.
        let square = match get_lsb_index(attack_mask) {
            Ok(v) => v,
            Err(m) => {
                // For now, just panic. This shouldn't happen here?
                panic!("{}", m);
            }
        };

        // Pop the bit.
        attack_mask = pop_bit(attack_mask, square as u64);

        // Make sure occupancy is on the board.
        if index & (1 << count) != 0 {
            occupancy |= 1 << square;
        }
    }

    return occupancy;
}

pub fn init_slider_attacks(
    is_bishop: bool,
    bishop_attacks: &mut Vec<Vec<u64>>,
    rook_attacks: &mut Vec<Vec<u64>>,
) {
    for square in 0..64 {
        let attack_mask;
        if is_bishop {
            attack_mask = constants::BISHOP_MASKED_ATTACKS[square];
        } else {
            attack_mask = constants::ROOK_MASKED_ATTACKS[square];
        }

        let relevant_bits_count = count_bits(attack_mask);
        let occupancy_indicies: usize = 1 << relevant_bits_count;
        for index in 0..occupancy_indicies {
            if is_bishop {
                let occupancy = set_occupancies(index, relevant_bits_count, attack_mask);
                let (temp, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
                let magic_index = (temp) >> 64 - constants::BISHOP_RELEVANT_BITS[square];
                bishop_attacks[square][magic_index as usize] =
                    dynamic_bishop_attacks(square as u64, occupancy);
            } else {
                let occupancy = set_occupancies(index, relevant_bits_count, attack_mask);
                let (temp, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
                let magic_index = (temp) >> 64 - constants::ROOK_RELEVANT_BITS[square];
                rook_attacks[square][magic_index as usize] =
                    dynamic_rook_attacks(square as u64, occupancy);
            }
        }
    }
}

// TODO: Figure out how to structure code that allows this to work... Pass in the constants?
pub fn get_bishop_attacks(c: &Constants, square: usize, mut occupancy: u64) -> u64 {
    occupancy &= constants::BISHOP_MASKED_ATTACKS[square];
    (occupancy, _) = occupancy.overflowing_mul(constants::BISHOP_MAGIC_NUMBERS[square]);
    occupancy >>= 64 - constants::BISHOP_RELEVANT_BITS[square];

    return c.bishop_attacks[square][occupancy as usize];
}

pub fn get_rook_attacks(c: &Constants, square: usize, mut occupancy: u64) -> u64 {
    occupancy &= constants::ROOK_MASKED_ATTACKS[square];
    (occupancy, _) = occupancy.overflowing_mul(constants::ROOK_MAGIC_NUMBERS[square]);
    occupancy >>= 64 - constants::ROOK_RELEVANT_BITS[square];

    return c.rook_attacks[square][occupancy as usize];
}

pub fn get_queen_attacks(c: &Constants, square: usize, occupancy: u64) -> u64 {
    return get_bishop_attacks(c, square, occupancy) | get_rook_attacks(c, square, occupancy);
}

// Should these be macros? Or something similar?
pub fn get_bit(bitboard: u64, square: u64) -> u64 {
    return bitboard & (1 << square);
}

pub fn set_bit(bitboard: u64, square: u64) -> u64 {
    return bitboard | (1 << square);
}

pub fn pop_bit(bitboard: u64, square: u64) -> u64 {
    if get_bit(bitboard, square) != 0 {
        return bitboard ^ (1 << square);
    } else {
        return bitboard;
    }
}

pub fn count_bits(mut bitboard: u64) -> usize {
    let mut bit_count = 0;

    while bitboard != 0 {
        bit_count += 1;

        // Reset the least significant bit, once per iteration until there are no active bits left.
        bitboard &= bitboard - 1;
    }

    return bit_count;
}

pub fn get_lsb_index(bitboard: u64) -> Result<usize, String> {
    // Below operations will not work on bitboard of `0`.
    if bitboard == 0 {
        return Err("Illegal index requested.".to_string());
    }

    // Get the position of the least-significant bit, using some bit magic!
    let lsb = bitboard & !bitboard + 1;

    // Subtract `1` to populate the trailing bits.
    let populated = lsb - 1;

    return Ok(count_bits(populated));
}

/*
    // This is the code we used to generate magic numbers. We don't need to run it again, but it should remain.
    struct MagicNumberHelper {
        pub state: u32,
    }

    impl MagicNumberHelper {
        pub fn new() -> Self {
            return MagicNumberHelper {
                state: 1804289383, // Seed for our Psuedo-RNG.
            }
        }

        fn get_random_number_u32(&mut self) -> u32 {
            // Get current state. This is our seed.
            let mut n = self.state;

            // XOR Shift Algorithm to get a random number.
            n ^= n << 13;
            n ^= n >> 17;
            n ^= n << 5;

            // Update the state.
            self.state = n;

            return n;
        }

        fn get_random_number_u64(&mut self) -> u64 {
            // Define some random numbers. We want the 16 bits from MSB1 side.
            let n1: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n2: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n3: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;
            let n4: u64 = (self.get_random_number_u32()) as u64 & 0xFFFF;

            // Return them with fanciness.
            return n1 | (n2 << 16) | (n3 << 32) | (n4 << 48);
        }

        fn get_magic_number(&mut self) -> u64 {
            return self.get_random_number_u64() & self.get_random_number_u64() & self.get_random_number_u64();
        }

        // Magic numbers?
        fn find_magic_number(&mut self, square: u64, relevant_bits: usize, is_bishop: bool) -> u64 {

            // Init occupancies, attack tables, and used attacks.
            let mut occupancies: [u64; 4096] = [0; 4096];
            let mut attacks: [u64; 4096] = [0; 4096];
            let mut used_attacks: [u64; 4096];

            // Init attack mask, either bishop or rook.
            let attack_mask: u64;
            if is_bishop {
                attack_mask = mask_bishop_attacks(square);
            } else {
                attack_mask = mask_rook_attacks(square);
            }

            // Init occupancy indicies.
            let occupancy_indicies: usize = 1 << relevant_bits;

            // Loop over occupancy indicies.
            for index in 0..occupancy_indicies {
                occupancies[index] = set_occupancies(index, relevant_bits, attack_mask);

                if is_bishop {
                    attacks[index] = dynamic_bishop_attacks(square, occupancies[index]);
                } else {
                    attacks[index] = dynamic_rook_attacks(square, occupancies[index]);
                }
            }

            // Now test for magic numbers. Should not take too long to run though!
            for _ in 0..1_000_000_000 {
                // Generate magic number candidate.
                let magic_number = self.get_magic_number();

                // This should be safe from overflow?
                let (temp, _) = attack_mask.overflowing_mul(magic_number);
                let to_check = temp & 0xFF00000000000000;

                // Go next if we don't have enough bits.
                if count_bits(to_check) < 6 {
                    continue;
                }

                // Clear out any used attacks from previous iteration.
                used_attacks = [0; 4096];
                let mut has_failed: bool = false;
                for index in 0..occupancy_indicies {

                    // Overflow safe?
                    let (temp, _) = occupancies[index].overflowing_mul(magic_number);
                    let magic_index = (temp >> (64 - relevant_bits)) as usize;

                    if used_attacks[magic_index] == 0 {
                        used_attacks[magic_index] = attacks[index];
                    } else if used_attacks[magic_index] != attacks[index] {
                        has_failed = true;
                        break;
                    }
                }

                if !has_failed {
                    return magic_number;
                }
            }

            panic!("Unable to find magic number, oh no!");
        }

        // This function will print all the magic numbers, then they can be copied for later use in other parts of the program.
        pub fn init_magic_numbers(&mut self, bishop_magic_numbers: &mut [u64; 64], rook_magic_numbers: &mut [u64; 64]) {
            // For each square on the board.
            for square in 0..64 {
                // Handle rooks.
                let n = self.find_magic_number(square, constants::ROOK_RELEVANT_BITS[square as usize], false);
                // println!("0x{:X}ULL", n);
                rook_magic_numbers[square as usize] = n;
            }

            println!("\nXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\n");

            // For each square on the board.
            for square in 0..64 {
                // Handle bishops.
                let n = self.find_magic_number(square, constants::BISHOP_RELEVANT_BITS[square as usize], true);
                // println!("0x{:X}ULL", n);
                bishop_magic_numbers[square as usize] = n;
            }
        }
    }
*/
