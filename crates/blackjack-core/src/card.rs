use std::fmt;

/// The four suits in a standard deck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    /// A one-character code used for asset names.
    pub const fn code(self) -> &'static str {
        match self {
            Suit::Clubs => "C",
            Suit::Diamonds => "D",
            Suit::Hearts => "H",
            Suit::Spades => "S",
        }
    }

    /// The traditional glyph for the suit.
    pub const fn symbol(self) -> &'static str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }

    pub const fn is_red(self) -> bool {
        matches!(self, Suit::Diamonds | Suit::Hearts)
    }
}

/// A card rank. Ordering follows rank strength, not blackjack value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    /// The value used when totalling a blackjack hand.
    pub const fn value(self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 1,
        }
    }

    /// The label printed on the card.
    pub const fn label(self) -> &'static str {
        match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        }
    }

    pub const fn is_ace(self) -> bool {
        matches!(self, Rank::Ace)
    }
}

/// A single playing card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    /// The blackjack value of the card, counting aces as one.
    pub const fn value(self) -> u8 {
        self.rank.value()
    }

    pub const fn is_ace(self) -> bool {
        self.rank.is_ace()
    }

    /// Two cards that can be split because they share a blackjack value.
    pub const fn same_value(self, other: Card) -> bool {
        self.rank.value() == other.rank.value()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank.label(), self.suit.symbol())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_values_match_blackjack() {
        assert_eq!(Rank::Ace.value(), 1);
        assert_eq!(Rank::King.value(), 10);
        assert_eq!(Rank::Ten.value(), 10);
        assert_eq!(Rank::Two.value(), 2);
    }

    #[test]
    fn same_value_groups_ten_value_cards() {
        let ten = Card::new(Rank::Ten, Suit::Hearts);
        let king = Card::new(Rank::King, Suit::Spades);
        let nine = Card::new(Rank::Nine, Suit::Clubs);

        assert!(ten.same_value(king));
        assert!(!ten.same_value(nine));
    }

    #[test]
    fn red_suits_are_red() {
        assert!(Suit::Hearts.is_red());
        assert!(Suit::Diamonds.is_red());
        assert!(!Suit::Clubs.is_red());
        assert!(!Suit::Spades.is_red());
    }
}
