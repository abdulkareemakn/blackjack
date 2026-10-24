//! Rules-driven blackjack engine.
//!
//! The engine is independent of the UI so rulesets, basic-strategy advice and
//! simulations can be built on the same tested core.

pub mod card;
pub mod game;
pub mod hand;
pub mod rules;
pub mod shoe;

pub use card::{Card, Rank, Suit};
pub use game::{BlackjackGame, GameError, Phase};
pub use hand::{Hand, HandOutcome, HandValue};
pub use rules::{BlackjackPayout, DoubleRule, Rules};
pub use shoe::Shoe;
