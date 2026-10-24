use std::fmt;

use crate::{
    card::Card,
    hand::{Hand, HandOutcome, HandValue},
    rules::Rules,
    shoe::Shoe,
};

/// Where a round currently is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for a wager.
    Betting,
    /// One of the player's hands is acting.
    PlayerTurn,
    /// The dealer has played and all hands have been paid.
    Settled,
}

/// Reasons an action was rejected by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    WrongPhase,
    BetTooLow,
    BetTooHigh,
    InsufficientFunds,
    ActionNotAllowed,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            GameError::WrongPhase => "That action is not available right now.",
            GameError::BetTooLow => "The bet is below the table minimum.",
            GameError::BetTooHigh => "The bet is above the table maximum.",
            GameError::InsufficientFunds => "There is not enough bankroll for that wager.",
            GameError::ActionNotAllowed => "That action is not allowed on this hand.",
        };
        f.write_str(message)
    }
}

impl std::error::Error for GameError {}

/// A complete blackjack round: betting, split hands, dealer play and payouts.
pub struct BlackjackGame {
    rules: Rules,
    shoe: Shoe,
    bankroll: u64,
    pending_bet: u64,
    dealer: Option<Hand>,
    hands: Vec<Hand>,
    active_hand: usize,
    phase: Phase,
    net_result: i64,
}

impl BlackjackGame {
    pub fn new(rules: Rules) -> Self {
        Self::with_seed(rules, rand::random())
    }

    pub fn with_seed(rules: Rules, seed: u64) -> Self {
        let shoe = Shoe::new(rules.decks, rules.penetration, seed);
        let bankroll = rules.starting_bankroll;
        Self::with_shoe(rules, shoe, bankroll)
    }

    pub fn with_shoe(rules: Rules, shoe: Shoe, bankroll: u64) -> Self {
        Self {
            rules,
            shoe,
            bankroll,
            pending_bet: 0,
            dealer: None,
            hands: Vec::new(),
            active_hand: 0,
            phase: Phase::Betting,
            net_result: 0,
        }
    }

    // ── State ──────────────────────────────────────────────────────────────

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn bankroll(&self) -> u64 {
        self.bankroll
    }

    pub fn pending_bet(&self) -> u64 {
        self.pending_bet
    }

    pub fn dealer(&self) -> Option<&Hand> {
        self.dealer.as_ref()
    }

    pub fn hands(&self) -> &[Hand] {
        &self.hands
    }

    pub fn active_hand_index(&self) -> usize {
        self.active_hand
    }

    pub fn active_hand(&self) -> Option<&Hand> {
        self.hands.get(self.active_hand)
    }

    /// Net cents won or lost on the last settled round. Positive values are
    /// profit, negative values are losses.
    pub fn net_result(&self) -> i64 {
        self.net_result
    }

    pub fn shoe_remaining(&self) -> usize {
        self.shoe.remaining()
    }

    pub fn shoe_total(&self) -> usize {
        self.shoe.total()
    }

    /// Total wagers committed to the hands in the current round.
    pub fn total_in_play(&self) -> u64 {
        self.hands.iter().map(Hand::bet).sum()
    }

    /// The hole card stays hidden only while the player is acting.
    pub fn dealer_hole_hidden(&self) -> bool {
        self.phase == Phase::PlayerTurn
    }

    pub fn dealer_upcard(&self) -> Option<Card> {
        self.dealer
            .as_ref()
            .and_then(|dealer| dealer.cards().first().copied())
    }

    // ── Betting ────────────────────────────────────────────────────────────

    pub fn can_add_chip(&self, amount: u64) -> bool {
        if self.phase != Phase::Betting || amount == 0 {
            return false;
        }
        let Some(new_bet) = self.pending_bet.checked_add(amount) else {
            return false;
        };
        new_bet <= self.rules.max_bet && new_bet <= self.bankroll
    }

    pub fn add_chip(&mut self, amount: u64) -> Result<(), GameError> {
        if self.phase != Phase::Betting {
            return Err(GameError::WrongPhase);
        }
        let new_bet = self
            .pending_bet
            .checked_add(amount)
            .ok_or(GameError::BetTooHigh)?;

        if new_bet > self.rules.max_bet {
            return Err(GameError::BetTooHigh);
        }
        if new_bet > self.bankroll {
            return Err(GameError::InsufficientFunds);
        }

        self.pending_bet = new_bet;
        Ok(())
    }

    pub fn set_bet(&mut self, amount: u64) -> Result<(), GameError> {
        if self.phase != Phase::Betting {
            return Err(GameError::WrongPhase);
        }
        if amount < self.rules.min_bet {
            return Err(GameError::BetTooLow);
        }
        if amount > self.rules.max_bet {
            return Err(GameError::BetTooHigh);
        }
        if amount > self.bankroll {
            return Err(GameError::InsufficientFunds);
        }
        self.pending_bet = amount;
        Ok(())
    }

    pub fn clear_bet(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Betting {
            return Err(GameError::WrongPhase);
        }
        self.pending_bet = 0;
        Ok(())
    }

    pub fn can_deal(&self) -> bool {
        self.phase == Phase::Betting
            && self.pending_bet >= self.rules.min_bet
            && self.pending_bet <= self.rules.max_bet
            && self.pending_bet <= self.bankroll
    }

    /// Add funds to the local table bankroll.
    pub fn add_funds(&mut self, amount: u64) {
        self.bankroll = self.bankroll.saturating_add(amount);
    }

    // ── Round flow ─────────────────────────────────────────────────────────

    pub fn deal(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Betting {
            return Err(GameError::WrongPhase);
        }
        if self.pending_bet < self.rules.min_bet {
            return Err(GameError::BetTooLow);
        }
        if self.pending_bet > self.rules.max_bet {
            return Err(GameError::BetTooHigh);
        }
        if self.pending_bet > self.bankroll {
            return Err(GameError::InsufficientFunds);
        }

        self.shoe.shuffle_if_needed();

        let bet = self.pending_bet;
        self.pending_bet = 0;
        self.bankroll -= bet;
        self.net_result = 0;
        self.active_hand = 0;
        self.hands = vec![Hand::new(bet)];
        self.dealer = Some(Hand::new(0));
        self.phase = Phase::PlayerTurn;

        let player_first = self.draw_card();
        self.hands[0].push_card(player_first);

        let dealer_up = self.draw_card();
        self.dealer
            .as_mut()
            .expect("dealer hand exists")
            .push_card(dealer_up);

        let player_second = self.draw_card();
        self.hands[0].push_card(player_second);

        let dealer_hole = self.draw_card();
        self.dealer
            .as_mut()
            .expect("dealer hand exists")
            .push_card(dealer_hole);

        let dealer_blackjack = self
            .dealer
            .as_ref()
            .is_some_and(|dealer| dealer.is_blackjack());

        if dealer_blackjack || self.hands[0].is_blackjack() {
            self.settle();
        }

        Ok(())
    }

    pub fn can_hit(&self) -> bool {
        self.phase == Phase::PlayerTurn
            && self
                .active_hand()
                .is_some_and(|hand| !hand.is_done() && !hand.is_blackjack())
    }

    pub fn hit(&mut self) -> Result<(), GameError> {
        if !self.can_hit() {
            return Err(GameError::ActionNotAllowed);
        }

        let card = self.draw_card();
        let hand = &mut self.hands[self.active_hand];
        hand.push_card(card);

        let value = hand.value();
        if value.is_bust() {
            hand.mark_busted();
        } else if value.total == 21 {
            hand.stand();
        }

        self.advance();
        Ok(())
    }

    pub fn can_stand(&self) -> bool {
        self.phase == Phase::PlayerTurn
            && self
                .active_hand()
                .is_some_and(|hand| !hand.is_done() && !hand.is_blackjack())
    }

    pub fn stand(&mut self) -> Result<(), GameError> {
        if !self.can_stand() {
            return Err(GameError::ActionNotAllowed);
        }

        self.hands[self.active_hand].stand();
        self.advance();
        Ok(())
    }

    pub fn can_double(&self) -> bool {
        if self.phase != Phase::PlayerTurn {
            return false;
        }
        let Some(hand) = self.active_hand() else {
            return false;
        };
        if hand.is_done() || hand.cards().len() != 2 || hand.is_blackjack() {
            return false;
        }
        if hand.is_from_split() && !self.rules.double_after_split {
            return false;
        }
        if self.bankroll < hand.bet() {
            return false;
        }
        self.rules.double_rule.allows(hand.value().total)
    }

    pub fn double_down(&mut self) -> Result<(), GameError> {
        if !self.can_double() {
            return Err(GameError::ActionNotAllowed);
        }

        let index = self.active_hand;
        let extra = self.hands[index].bet();
        self.bankroll -= extra;

        let card = self.draw_card();
        let hand = &mut self.hands[index];
        hand.double_bet();
        hand.push_card(card);

        if hand.value().is_bust() {
            hand.mark_busted();
        }
        hand.stand();

        self.advance();
        Ok(())
    }

    pub fn can_split(&self) -> bool {
        if self.phase != Phase::PlayerTurn {
            return false;
        }
        let Some(hand) = self.active_hand() else {
            return false;
        };
        if hand.is_done() || hand.cards().len() != 2 {
            return false;
        }
        let [first, second] = [hand.cards()[0], hand.cards()[1]];
        if !first.same_value(second) {
            return false;
        }
        if self.hands.len() >= self.rules.max_split_hands as usize {
            return false;
        }
        if self.bankroll < hand.bet() {
            return false;
        }
        if first.rank.is_ace() && hand.is_from_split() && !self.rules.resplit_aces {
            return false;
        }
        true
    }

    pub fn split(&mut self) -> Result<(), GameError> {
        if !self.can_split() {
            return Err(GameError::ActionNotAllowed);
        }

        let index = self.active_hand;
        let (first_card, second_card) = self.hands[index].split_pair();
        let bet = self.hands[index].bet();
        self.bankroll -= bet;

        let mut first = Hand::from_split(first_card, bet);
        first.push_card(self.draw_card());

        let mut second = Hand::from_split(second_card, bet);
        second.push_card(self.draw_card());

        let aces = first_card.rank.is_ace();
        if aces && self.rules.split_aces_one_card {
            first.stand();
            second.stand();
        } else {
            if first.value().total == 21 {
                first.stand();
            }
            if second.value().total == 21 {
                second.stand();
            }
        }

        self.hands[index] = first;
        self.hands.insert(index + 1, second);
        self.advance();
        Ok(())
    }

    pub fn next_round(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Settled {
            return Err(GameError::WrongPhase);
        }
        self.dealer = None;
        self.hands.clear();
        self.active_hand = 0;
        self.pending_bet = 0;
        self.net_result = 0;
        self.phase = Phase::Betting;
        Ok(())
    }

    // ── Internals ──────────────────────────────────────────────────────────

    fn draw_card(&mut self) -> Card {
        self.shoe
            .draw()
            .expect("the shoe is shuffled before a round and always has cards")
    }

    fn advance(&mut self) {
        let mut index = self.active_hand;

        loop {
            while index < self.hands.len() && self.hands[index].is_done() {
                index += 1;
            }

            if index >= self.hands.len() {
                self.play_dealer();
                return;
            }

            if self.hands[index].value().total == 21 {
                self.hands[index].stand();
                index += 1;
                continue;
            }

            self.active_hand = index;
            return;
        }
    }

    fn play_dealer(&mut self) {
        let any_live_hand = self.hands.iter().any(|hand| !hand.is_busted());

        if any_live_hand {
            loop {
                let should_hit = self.dealer.as_ref().is_some_and(|dealer| {
                    let value = dealer.value();
                    value.total < 17
                        || (value.soft && !self.rules.dealer_stands_soft_17)
                });

                if !should_hit {
                    break;
                }

                let card = self.draw_card();
                self.dealer
                    .as_mut()
                    .expect("dealer hand exists")
                    .push_card(card);
            }
        }

        self.settle();
    }

    fn settle(&mut self) {
        let dealer_value = self
            .dealer
            .as_ref()
            .map(Hand::value)
            .unwrap_or(HandValue {
                total: 0,
                soft: false,
            });
        let dealer_blackjack = self
            .dealer
            .as_ref()
            .is_some_and(|dealer| dealer.is_blackjack());
        let dealer_busted = dealer_value.is_bust();
        let payout_rule = self.rules.blackjack_payout;

        let mut bankroll = self.bankroll;
        let mut net = 0_i64;

        for hand in &mut self.hands {
            let bet = hand.bet();
            let player_value = hand.value();
            let outcome = if hand.is_busted() {
                HandOutcome::Loss
            } else if dealer_blackjack {
                if hand.is_blackjack() {
                    HandOutcome::Push
                } else {
                    HandOutcome::Loss
                }
            } else if hand.is_blackjack() {
                HandOutcome::Blackjack
            } else if dealer_busted || player_value.total > dealer_value.total {
                HandOutcome::Win
            } else if player_value.total == dealer_value.total {
                HandOutcome::Push
            } else {
                HandOutcome::Loss
            };

            let payout = outcome.payout(bet, |wager| payout_rule.profit(wager));
            bankroll = bankroll.saturating_add(payout);
            net += payout as i64 - bet as i64;
            hand.settle(outcome, payout);
        }

        self.bankroll = bankroll;
        self.net_result = net;
        self.phase = Phase::Settled;
    }
}

impl Default for BlackjackGame {
    fn default() -> Self {
        Self::new(Rules::default())
    }
}
