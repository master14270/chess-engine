pub mod castle_sides;
pub mod constants;
pub mod coord;
pub mod g; // Something is wrong with rust-analyzer. This is the only way it will pick up the changes right now.
pub mod game_states;
pub mod my_move;
pub mod piece;
pub mod piece_type;
pub mod tests;

// use crate::g::Game;
use crate::tests::run_all_tests;

fn main() {
    // let mut game: Game = g::Game::default();
    // game.import_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // //game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR w KQkq - 0 1");
    // //game.import_fen("rnbqkbnr/8/8/8/8/8/8/RNBQKBNR b KQkq - 0 1");
    // game.print_board();
    // game.print_all_legal_moves();

    // Run testing code.
    run_all_tests();
}
