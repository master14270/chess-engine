// This will have all the struct definitions we will need to run the bot.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserBasic {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub rating: u32,
    pub provisional: bool,
}

// Implement the Default trait
impl Default for UserBasic {
    fn default() -> Self {
        UserBasic {
            id: String::new(),
            name: String::new(),
            title: None,
            rating: 0,
            provisional: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub r#type: String,
    pub moves: String,
    pub wtime: u32,
    pub btime: u32,
    pub winc: u32,
    pub binc: u32,
    pub status: String,
}

impl GameState {
    pub fn moves_to_vec(&self) -> Vec<String> {
        if self.moves == "" {
            return vec![];
        }

        // Split moves on the space.
        let mut moves_vec: Vec<String> = vec![];
        let mut sp = self.moves.split(" ");

        loop {
            // Read moves until we are all done.
            let m = match sp.next() {
                Some(s) => s.to_string(),
                None => break,
            };

            moves_vec.push(m);
        }

        return moves_vec;
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            r#type: String::new(),
            moves: String::new(),
            wtime: 0,
            btime: 0,
            winc: 0,
            binc: 0,
            status: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameFull {
    pub r#type: String,
    pub id: String,
    pub white: UserBasic,
    pub black: UserBasic,
    pub state: GameState,

    // Lichess API gives us CAMEL CASE. So we fix it.
    #[serde(rename = "initialFen")]
    pub initial_fen: String,
}

// Implement the Default trait for Piece
impl Default for GameFull {
    fn default() -> Self {
        GameFull {
            r#type: String::new(),
            id: String::new(),
            white: UserBasic::default(),
            black: UserBasic::default(),
            state: GameState::default(),
            initial_fen: String::new(),
        }
    }
}
