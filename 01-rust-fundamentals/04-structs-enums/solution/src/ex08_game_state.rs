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
        GameState::Menu
    }

    /// Menu -> Playing. Also restarts after GameOver.
    pub fn start_game(&mut self) {
        if matches!(self, GameState::Menu | GameState::GameOver { .. }) {
            *self = GameState::Playing {
                level: 1,
                score: 0,
                lives: 3,
            };
        }
    }

    /// Playing -> Paused.
    ///
    /// Matching on `*self` copies the u32/u8 fields out (they are Copy), so
    /// no borrow of `self` is alive when we assign the new state.
    pub fn pause(&mut self) {
        if let GameState::Playing {
            level,
            score,
            lives,
        } = *self
        {
            *self = GameState::Paused {
                level,
                score,
                lives,
            };
        }
    }

    /// Paused -> Playing.
    pub fn resume(&mut self) {
        if let GameState::Paused {
            level,
            score,
            lives,
        } = *self
        {
            *self = GameState::Playing {
                level,
                score,
                lives,
            };
        }
    }

    /// Only while Playing. Matching on `self` (a `&mut GameState`) binds
    /// `score` as a `&mut u32` pointing *into* the enum, so we can update the
    /// field in place.
    pub fn add_score(&mut self, points: u32) {
        if let GameState::Playing { score, .. } = self {
            *score = score.saturating_add(points);
        }
    }

    /// Playing -> Playing with one fewer life, or -> GameOver at zero.
    pub fn lose_life(&mut self) {
        if let GameState::Playing {
            level,
            score,
            lives,
        } = *self
        {
            *self = if lives <= 1 {
                GameState::GameOver { final_score: score }
            } else {
                GameState::Playing {
                    level,
                    score,
                    lives: lives - 1,
                }
            };
        }
    }

    /// Playing -> Playing at the next level, with LEVEL_BONUS * level points.
    pub fn next_level(&mut self) {
        if let GameState::Playing { level, score, .. } = self {
            *score = score.saturating_add(LEVEL_BONUS * *level);
            *level += 1;
        }
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self, GameState::GameOver { .. })
    }

    pub fn display(&self) -> String {
        match self {
            GameState::Menu => "=== MAIN MENU ===".to_string(),
            GameState::Playing {
                level,
                score,
                lives,
            } => {
                format!("Level: {level} | Score: {score} | Lives: {lives}")
            }
            GameState::Paused {
                level,
                score,
                lives,
            } => {
                format!("PAUSED - Level: {level} | Score: {score} | Lives: {lives}")
            }
            GameState::GameOver { final_score } => {
                format!("GAME OVER - Final Score: {final_score}")
            }
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run() {
    let mut game = GameState::new();
    println!("{}", game.display());
    game.start_game();
    println!("{}", game.display());
    game.add_score(100);
    println!("{}", game.display());
    game.pause();
    println!("{}", game.display());
    game.add_score(1_000); // ignored: paused
    game.resume();
    game.add_score(50);
    println!("{}", game.display());
    game.next_level();
    println!("{}", game.display());
    game.lose_life();
    game.lose_life();
    game.lose_life();
    println!("{}", game.display());
    assert!(game.is_game_over());
}
