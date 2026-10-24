use crate::card::Card;

/// The total value of a hand, including whether an ace is counted as eleven.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandValue {
    pub total: u8,
    pub soft: bool,
}

impl HandValue {
    /// Calculate a hand value, counting aces as one and upgrading one ace to
    /// eleven whenever that does not bust the hand.
    pub fn from_cards(cards: &[Card]) -> Self {
        let aces = cards.iter().filter(|card| card.is_ace()).count();
        let total: u8 = cards.iter().map(|card| card.value()).sum();

        if aces > 0 && total + 10 <= 21 {
            Self {
                total: total + 10,
                soft: true,
            }
        } else {
            Self {
                total,
                soft: false,
            }
        }
    }

    pub fn is_bust(self) -> bool {
        self.total > 21
    }

    pub fn is_blackjack_total(self) -> bool {
        self.total == 21
    }
}

impl Default for HandValue {
    fn default() -> Self {
        Self {
            total: 0,
            soft: false,
        }
    }
}

/// How a finished hand compared with the dealer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandOutcome {
    Blackjack,
    Win,
    Push,
    Loss,
}

impl HandOutcome {
    pub const fn label(self) -> &'static str {
        match self {
            HandOutcome::Blackjack => "BLACKJACK",
            HandOutcome::Win => "WIN",
            HandOutcome::Push => "PUSH",
            HandOutcome::Loss => "LOSE",
        }
    }

    /// Payout in cents, including the returned stake. The wager has already
    /// been deducted from the bankroll by the time this is paid.
    pub fn payout(self, bet: u64, blackjack_profit: impl Fn(u64) -> u64) -> u64 {
        match self {
            HandOutcome::Blackjack => bet + blackjack_profit(bet),
            HandOutcome::Win => bet.saturating_mul(2),
            HandOutcome::Push => bet,
            HandOutcome::Loss => 0,
        }
    }
}

/// A player or dealer hand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hand {
    cards: Vec<Card>,
    bet: u64,
    doubled: bool,
    from_split: bool,
    stood: bool,
    busted: bool,
    outcome: Option<HandOutcome>,
    payout: u64,
}

impl Hand {
    pub(crate) fn new(bet: u64) -> Self {
        Self {
            cards: Vec::with_capacity(6),
            bet,
            doubled: false,
            from_split: false,
            stood: false,
            busted: false,
            outcome: None,
            payout: 0,
        }
    }

    pub(crate) fn from_split(card: Card, bet: u64) -> Self {
        let mut hand = Self::new(bet);
        hand.from_split = true;
        hand.cards.push(card);
        hand
    }

    pub(crate) fn push_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn bet(&self) -> u64 {
        self.bet
    }

    pub fn is_doubled(&self) -> bool {
        self.doubled
    }

    pub fn is_from_split(&self) -> bool {
        self.from_split
    }

    pub fn is_stood(&self) -> bool {
        self.stood
    }

    pub fn is_busted(&self) -> bool {
        self.busted
    }

    pub fn is_done(&self) -> bool {
        self.stood || self.busted
    }

    pub fn value(&self) -> HandValue {
        HandValue::from_cards(&self.cards)
    }

    /// A natural blackjack is a two-card 21 on the original hand. A 21 made
    /// after splitting is an ordinary 21 and pays even money.
    pub fn is_blackjack(&self) -> bool {
        !self.from_split && self.cards.len() == 2 && self.value().total == 21
    }

    pub fn outcome(&self) -> Option<HandOutcome> {
        self.outcome
    }

    pub fn payout(&self) -> u64 {
        self.payout
    }

    pub(crate) fn stand(&mut self) {
        self.stood = true;
    }

    pub(crate) fn mark_busted(&mut self) {
        self.busted = true;
    }

    pub(crate) fn double_bet(&mut self) {
        self.bet = self.bet.saturating_mul(2);
        self.doubled = true;
    }

    pub(crate) fn settle(&mut self, outcome: HandOutcome, payout: u64) {
        self.outcome = Some(outcome);
        self.payout = payout;
    }

    /// The two cards used to create split hands. Only valid on a two-card hand.
    pub(crate) fn split_pair(&self) -> (Card, Card) {
        debug_assert_eq!(self.cards.len(), 2, "only two-card hands can be split");
        (self.cards[0], self.cards[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Rank, Suit};

    fn cards(ranks: &[Rank], suit: Suit) -> Vec<Card> {
        ranks
            .iter()
            .map(|rank| Card::new(*rank, suit))
            .collect()
    }

    #[test]
    fn hard_total_has_no_ace_eleven() {
        let hand = cards(&[Rank::King, Rank::Seven], Suit::Spades);
        assert_eq!(
            HandValue::from_cards(&hand),
            HandValue {
                total: 17,
                soft: false
            }
        );
    }

    #[test]
    fn soft_total_counts_an_ace_as_eleven() {
        let hand = cards(&[Rank::Ace, Rank::Six], Suit::Hearts);
        assert_eq!(
            HandValue::from_cards(&hand),
            HandValue {
                total: 17,
                soft: true
            }
        );
    }

    #[test]
    fn ace_falls_back_to_one_when_eleven_would_bust() {
        let hand = cards(&[Rank::Ace, Rank::King, Rank::Queen], Suit::Clubs);
        assert_eq!(
            HandValue::from_cards(&hand),
            HandValue {
                total: 21,
                soft: false
            }
        );
    }

    #[test]
    fn multiple_aces_stay_playable() {
        let hand = cards(&[Rank::Ace, Rank::Ace, Rank::Nine], Suit::Diamonds);
        assert_eq!(
            HandValue::from_cards(&hand),
            HandValue {
                total: 21,
                soft: true
            }
        );
    }

    #[test]
    fn bust_is_over_twenty_one() {
        let hand = cards(&[Rank::King, Rank::Queen, Rank::Five], Suit::Spades);
        assert!(HandValue::from_cards(&hand).is_bust());
    }

    #[test]
    fn natural_blackjack_requires_an_unsplit_two_card_hand() {
        let mut hand = Hand::new(1_000);
        hand.push_card(Card::new(Rank::Ace, Suit::Spades));
        hand.push_card(Card::new(Rank::King, Suit::Hearts));
        assert!(hand.is_blackjack());

        let split = Hand::from_split(Card::new(Rank::Ace, Suit::Spades), 1_000);
        let mut split = split;
        split.push_card(Card::new(Rank::King, Suit::Hearts));
        assert!(!split.is_blackjack());
    }

    #[test]
    fn payout_includes_returned_stake() {
        let blackjack = |bet| bet * 3 / 2;
        assert_eq!(HandOutcome::Blackjack.payout(1_000, blackjack), 2_500);
        assert_eq!(HandOutcome::Win.payout(1_000, blackjack), 2_000);
        assert_eq!(HandOutcome::Push.payout(1_000, blackjack), 1_000);
        assert_eq!(HandOutcome::Loss.payout(1_000, blackjack), 0);
    }
}
