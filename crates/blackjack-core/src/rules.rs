/// How a natural blackjack is paid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlackjackPayout {
    pub profit_numerator: u32,
    pub profit_denominator: u32,
}

impl BlackjackPayout {
    pub const THREE_TO_TWO: Self = Self {
        profit_numerator: 3,
        profit_denominator: 2,
    };

    pub const SIX_TO_FIVE: Self = Self {
        profit_numerator: 6,
        profit_denominator: 5,
    };

    /// The profit on a winning natural, not including the returned wager.
    pub fn profit(self, bet: u64) -> u64 {
        if self.profit_denominator == 0 {
            return 0;
        }
        bet.saturating_mul(self.profit_numerator as u64) / self.profit_denominator as u64
    }
}

/// Which starting hands may double down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoubleRule {
    AnyTwo,
    NineToEleven,
    TenToEleven,
}

impl DoubleRule {
    pub fn allows(self, total: u8) -> bool {
        match self {
            DoubleRule::AnyTwo => true,
            DoubleRule::NineToEleven => (9..=11).contains(&total),
            DoubleRule::TenToEleven => (10..=11).contains(&total),
        }
    }
}

/// House rules for a blackjack table. Rules are kept separate from the engine
/// so future variants and their strategy charts can share one implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct Rules {
    pub name: &'static str,
    pub decks: u8,
    /// Fraction of the shoe dealt before the cut card. `0.75` reshuffles when
    /// roughly a quarter of the cards remain.
    pub penetration: f32,
    pub blackjack_payout: BlackjackPayout,
    pub dealer_stands_soft_17: bool,
    pub double_rule: DoubleRule,
    pub double_after_split: bool,
    pub max_split_hands: u8,
    pub split_aces_one_card: bool,
    pub resplit_aces: bool,
    pub dealer_peeks: bool,
    pub min_bet: u64,
    pub max_bet: u64,
    pub starting_bankroll: u64,
}

impl Rules {
    /// Six decks, dealer stands on soft 17, blackjack pays 3:2, double on any
    /// two cards including after a split, and up to four hands.
    pub fn vegas_classic() -> Self {
        Self {
            name: "Vegas Classic",
            decks: 6,
            penetration: 0.75,
            blackjack_payout: BlackjackPayout::THREE_TO_TWO,
            dealer_stands_soft_17: true,
            double_rule: DoubleRule::AnyTwo,
            double_after_split: true,
            max_split_hands: 4,
            split_aces_one_card: true,
            resplit_aces: false,
            dealer_peeks: true,
            min_bet: 500,
            max_bet: 50_000,
            starting_bankroll: 100_000,
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::vegas_classic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_to_two_profit_is_exact_for_chip_multiples() {
        assert_eq!(BlackjackPayout::THREE_TO_TWO.profit(500), 750);
        assert_eq!(BlackjackPayout::THREE_TO_TWO.profit(1_000), 1_500);
    }

    #[test]
    fn double_rules_limit_starting_totals() {
        assert!(DoubleRule::AnyTwo.allows(5));
        assert!(DoubleRule::NineToEleven.allows(9));
        assert!(!DoubleRule::NineToEleven.allows(8));
        assert!(DoubleRule::TenToEleven.allows(11));
        assert!(!DoubleRule::TenToEleven.allows(9));
    }
}
