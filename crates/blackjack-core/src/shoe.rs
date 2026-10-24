use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

use crate::card::{Card, Rank, Suit};

/// A multi-deck shoe with a cut card and a seeded shuffle.
pub struct Shoe {
    decks: u8,
    cards: Vec<Card>,
    total: usize,
    cut_card: usize,
    rng: StdRng,
}

impl Shoe {
    /// Create and shuffle a new shoe.
    ///
    /// `penetration` is the fraction of the shoe dealt before the cut card;
    /// `0.75` means the shoe is reshuffled when about a quarter remains.
    pub fn new(decks: u8, penetration: f32, seed: u64) -> Self {
        let decks = decks.max(1);
        let total = decks as usize * 52;
        let penetration = penetration.clamp(0.0, 1.0);
        let cut_card = total.saturating_sub((total as f32 * penetration).round() as usize);

        let mut shoe = Self {
            decks,
            cards: Vec::with_capacity(total),
            total,
            cut_card,
            rng: StdRng::seed_from_u64(seed),
        };
        shoe.shuffle();
        shoe
    }

    /// Build a shoe from a scripted draw order. The first card in `cards` is
    /// the first card drawn. Useful for deterministic tests.
    pub fn stacked(cards: Vec<Card>) -> Self {
        let total = cards.len();
        Self {
            decks: 1,
            cards: cards.into_iter().rev().collect(),
            total,
            cut_card: 0,
            rng: StdRng::seed_from_u64(0),
        }
    }

    /// Rebuild and shuffle the full shoe.
    pub fn shuffle(&mut self) {
        self.cards.clear();
        self.cards.reserve(self.total);

        for _ in 0..self.decks {
            for suit in Suit::ALL {
                for rank in Rank::ALL {
                    self.cards.push(Card::new(rank, suit));
                }
            }
        }

        self.cards.shuffle(&mut self.rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn decks(&self) -> u8 {
        self.decks
    }

    /// Whether the cut card has been reached.
    pub fn needs_shuffle(&self) -> bool {
        self.total > 0 && self.cards.len() <= self.cut_card
    }

    pub fn shuffle_if_needed(&mut self) {
        if self.needs_shuffle() {
            self.shuffle();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn six_deck_shoe_contains_every_card_six_times() {
        let mut shoe = Shoe::new(6, 0.75, 42);
        assert_eq!(shoe.total(), 312);
        assert_eq!(shoe.remaining(), 312);

        let mut counts = HashMap::new();
        while let Some(card) = shoe.draw() {
            *counts.entry((card.rank, card.suit)).or_insert(0_u8) += 1;
        }

        assert_eq!(counts.len(), 52);
        assert!(counts.values().all(|count| *count == 6));
        assert_eq!(shoe.remaining(), 0);
    }

    #[test]
    fn cut_card_requests_a_shuffle() {
        let mut shoe = Shoe::new(6, 0.75, 7);
        assert!(!shoe.needs_shuffle());

        while shoe.remaining() > 78 {
            shoe.draw();
        }

        assert_eq!(shoe.remaining(), 78);
        assert!(shoe.needs_shuffle());
        shoe.shuffle_if_needed();
        assert_eq!(shoe.remaining(), 312);
        assert!(!shoe.needs_shuffle());
    }

    #[test]
    fn stacked_shoe_pops_cards_in_draw_order() {
        let first = Card::new(Rank::Ace, Suit::Spades);
        let second = Card::new(Rank::King, Suit::Hearts);
        let mut shoe = Shoe::stacked(vec![first, second]);

        assert_eq!(shoe.draw(), Some(first));
        assert_eq!(shoe.draw(), Some(second));
        assert_eq!(shoe.draw(), None);
    }
}
