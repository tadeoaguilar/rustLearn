//! Exercise 8: Complex Enum - Game State.
//!
//! An enum-based state machine. Notice what *can't* happen: there is no
//! `score` field in `Menu`, so you can't accidentally read a score before the
//! game starts. With a struct full of Option fields and an `is_paused: bool`,
//! nonsense combinations like "paused, but in the menu" would be representable.
//!
//! Every transition is a no-op from a state where it makes no sense (you can't
//! pause the menu). An alternative design returns `Result<(), InvalidTransition>`.

/// Points awarded for finishing a level, multiplied by the level number.
pub const LEVEL_BONUS: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing { level: u32, score: u32, lives: u8 },
    Paused { level: u32, score: u32, lives: u8 },
    GameOver { final_score: u32 },
}

impl GameState {
    pub fn new() -> Self {
        todo!("Exercise 8")
    }

    /// Menu -> Playing. Also restarts after GameOver.
    pub fn start_game(&mut self) {
        todo!("Exercise 8")
    }

    /// Playing -> Paused.
    ///
    /// Matching on `*self` copies the u32/u8 fields out (they are Copy), so
    /// no borrow of `self` is alive when we assign the new state.
    pub fn pause(&mut self) {
        todo!("Exercise 8")
    }

    /// Paused -> Playing.
    pub fn resume(&mut self) {
        todo!("Exercise 8")
    }

    /// Only while Playing. Matching on `self` (a `&mut GameState`) binds
    /// `score` as a `&mut u32` pointing *into* the enum, so we can update the
    /// field in place.
    pub fn add_score(&mut self, points: u32) {
        todo!("Exercise 8")
    }

    /// Playing -> Playing with one fewer life, or -> GameOver at zero.
    pub fn lose_life(&mut self) {
        todo!("Exercise 8")
    }

    /// Playing -> Playing at the next level, with LEVEL_BONUS * level points.
    pub fn next_level(&mut self) {
        todo!("Exercise 8")
    }

    pub fn is_game_over(&self) -> bool {
        todo!("Exercise 8")
    }

    pub fn display(&self) -> String {
        todo!("Exercise 8")
    }
}

impl Default for GameState {
    fn default() -> Self {
        todo!("Exercise 8")
    }
}

pub fn run() {
    todo!("Exercise 8")
}
