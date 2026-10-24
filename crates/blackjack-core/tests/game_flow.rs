use blackjack_core::{
    BlackjackGame, Card, GameError, HandOutcome, Phase, Rank, Rules, Shoe, Suit,
};

type Script = [(Rank, Suit)];

fn stacked_game(rules: Rules, script: &Script, bankroll: u64) -> BlackjackGame {
    let cards = script
        .iter()
        .map(|(rank, suit)| Card::new(*rank, *suit))
        .collect();
    BlackjackGame::with_shoe(rules, Shoe::stacked(cards), bankroll)
}

fn standard_game(script: &Script) -> BlackjackGame {
    stacked_game(Rules::vegas_classic(), script, 10_000)
}

#[test]
fn natural_blackjack_pays_three_to_two() {
    let script = [
        (Rank::Ace, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::King, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert_eq!(game.hands().len(), 1);
    assert!(game.hands()[0].is_blackjack());
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Blackjack));
    assert_eq!(game.hands()[0].payout(), 2_500);
    assert_eq!(game.bankroll(), 11_500);
    assert_eq!(game.net_result(), 1_500);
}

#[test]
fn five_dollar_blackjack_can_pay_fifty_cents_of_profit() {
    let script = [
        (Rank::Ace, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::King, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(500).unwrap();
    game.deal().unwrap();

    assert_eq!(game.hands()[0].payout(), 1_250);
    assert_eq!(game.bankroll(), 10_750);
    assert_eq!(game.net_result(), 750);
}

#[test]
fn both_naturals_push() {
    let script = [
        (Rank::Ace, Suit::Spades),
        (Rank::Ace, Suit::Clubs),
        (Rank::King, Suit::Hearts),
        (Rank::Queen, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Push));
    assert_eq!(game.hands()[0].payout(), 1_000);
    assert_eq!(game.bankroll(), 10_000);
    assert_eq!(game.net_result(), 0);
}

#[test]
fn dealer_peeks_for_blackjack_and_ends_the_round() {
    let script = [
        (Rank::Ten, Suit::Spades),
        (Rank::Ace, Suit::Clubs),
        (Rank::Nine, Suit::Hearts),
        (Rank::King, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert!(game.dealer().unwrap().is_blackjack());
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Loss));
    assert_eq!(game.bankroll(), 9_000);
    assert_eq!(game.net_result(), -1_000);
    assert!(!game.dealer_hole_hidden());
}

#[test]
fn player_wins_when_dealer_busts() {
    let script = [
        (Rank::Ten, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Eight, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Ten, Suit::Hearts),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    assert_eq!(game.phase(), Phase::PlayerTurn);
    assert!(game.dealer_hole_hidden());

    game.stand().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert_eq!(game.dealer().unwrap().value().total, 26);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Win));
    assert_eq!(game.hands()[0].payout(), 2_000);
    assert_eq!(game.bankroll(), 11_000);
    assert_eq!(game.net_result(), 1_000);
}

#[test]
fn equal_totals_push() {
    let script = [
        (Rank::Ten, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Eight, Suit::Hearts),
        (Rank::Nine, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    game.stand().unwrap();

    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Push));
    assert_eq!(game.bankroll(), 10_000);
    assert_eq!(game.net_result(), 0);
}

#[test]
fn dealer_stands_on_soft_seventeen() {
    let script = [
        (Rank::Ten, Suit::Spades),
        (Rank::Ace, Suit::Clubs),
        (Rank::Eight, Suit::Hearts),
        (Rank::Six, Suit::Diamonds),
        (Rank::Four, Suit::Hearts),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    game.stand().unwrap();

    let dealer = game.dealer().unwrap();
    assert_eq!(dealer.cards().len(), 2);
    assert_eq!(dealer.value().total, 17);
    assert!(dealer.value().soft);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Win));
}

#[test]
fn a_rule_variant_can_make_the_dealer_hit_soft_seventeen() {
    let mut rules = Rules::vegas_classic();
    rules.dealer_stands_soft_17 = false;

    let script = [
        (Rank::Ten, Suit::Spades),
        (Rank::Ace, Suit::Clubs),
        (Rank::Eight, Suit::Hearts),
        (Rank::Six, Suit::Diamonds),
        (Rank::Four, Suit::Hearts),
    ];
    let mut game = stacked_game(rules, &script, 10_000);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    game.stand().unwrap();

    let dealer = game.dealer().unwrap();
    assert_eq!(dealer.cards().len(), 3);
    assert_eq!(dealer.value().total, 21);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Loss));
}

#[test]
fn double_down_doubles_the_wager_and_draws_once() {
    let script = [
        (Rank::Five, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Six, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Ten, Suit::Hearts),
        (Rank::Ten, Suit::Clubs),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    assert!(game.can_double());

    game.double_down().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert_eq!(game.hands()[0].bet(), 2_000);
    assert!(game.hands()[0].is_doubled());
    assert_eq!(game.hands()[0].cards().len(), 3);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Win));
    assert_eq!(game.hands()[0].payout(), 4_000);
    assert_eq!(game.bankroll(), 12_000);
    assert_eq!(game.net_result(), 2_000);
}

#[test]
fn split_creates_two_hands_and_double_after_split_is_allowed() {
    let script = [
        (Rank::Eight, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Eight, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Ten, Suit::Clubs),
        (Rank::Two, Suit::Diamonds),
        (Rank::Six, Suit::Hearts),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    assert!(game.can_split());

    game.split().unwrap();

    assert_eq!(game.hands().len(), 2);
    assert_eq!(game.hands()[0].bet(), 1_000);
    assert_eq!(game.hands()[1].bet(), 1_000);
    assert_eq!(game.bankroll(), 8_000);
    assert!(game.can_double());

    game.stand().unwrap();
    game.stand().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Win));
    assert_eq!(game.hands()[1].outcome(), Some(HandOutcome::Win));
    assert_eq!(game.hands()[0].payout(), 2_000);
    assert_eq!(game.hands()[1].payout(), 2_000);
    assert_eq!(game.bankroll(), 12_000);
    assert_eq!(game.net_result(), 2_000);
}

#[test]
fn split_aces_receive_one_card_each_and_auto_stand() {
    let script = [
        (Rank::Ace, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Ace, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::King, Suit::Diamonds),
        (Rank::Queen, Suit::Clubs),
        (Rank::Ten, Suit::Clubs),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    game.split().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert_eq!(game.hands().len(), 2);
    assert_eq!(game.hands()[0].cards().len(), 2);
    assert_eq!(game.hands()[1].cards().len(), 2);
    assert!(!game.hands()[0].is_blackjack());
    assert!(!game.hands()[1].is_blackjack());
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Win));
    assert_eq!(game.hands()[1].outcome(), Some(HandOutcome::Win));
    assert_eq!(game.hands()[0].payout(), 2_000);
    assert_eq!(game.hands()[1].payout(), 2_000);
    assert_eq!(game.net_result(), 2_000);
}

#[test]
fn splitting_stops_at_four_hands() {
    let script = [
        (Rank::Eight, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Eight, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Eight, Suit::Diamonds),
        (Rank::Two, Suit::Clubs),
        (Rank::Eight, Suit::Clubs),
        (Rank::Three, Suit::Clubs),
        (Rank::Four, Suit::Diamonds),
        (Rank::Five, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    game.split().unwrap();
    assert_eq!(game.hands().len(), 2);
    assert!(game.can_split());

    game.split().unwrap();
    assert_eq!(game.hands().len(), 3);
    assert!(game.can_split());

    game.split().unwrap();
    assert_eq!(game.hands().len(), 4);
    assert_eq!(game.phase(), Phase::PlayerTurn);
    assert!(!game.can_split());
    assert_eq!(game.bankroll(), 6_000);
}

#[test]
fn busted_player_does_not_make_the_dealer_draw() {
    let script = [
        (Rank::Ten, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::Six, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
        (Rank::Ten, Suit::Hearts),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    game.hit().unwrap();

    assert_eq!(game.phase(), Phase::Settled);
    assert!(game.hands()[0].is_busted());
    assert_eq!(game.dealer().unwrap().cards().len(), 2);
    assert_eq!(game.hands()[0].outcome(), Some(HandOutcome::Loss));
    assert_eq!(game.bankroll(), 9_000);
}

#[test]
fn bets_are_validated_against_limits_and_bankroll() {
    let mut game = standard_game(&[]);

    assert_eq!(game.set_bet(100), Err(GameError::BetTooLow));
    assert_eq!(game.set_bet(100_000), Err(GameError::BetTooHigh));

    let mut limited = stacked_game(Rules::vegas_classic(), &[], 1_000);
    assert_eq!(limited.set_bet(2_000), Err(GameError::InsufficientFunds));
    limited.clear_bet().unwrap();
    assert_eq!(limited.add_chip(1_500), Err(GameError::InsufficientFunds));
}

#[test]
fn deal_requires_a_bet_within_the_table_limits() {
    let mut game = standard_game(&[]);

    assert_eq!(game.deal(), Err(GameError::BetTooLow));

    game.add_chip(400).unwrap();
    assert_eq!(game.deal(), Err(GameError::BetTooLow));

    game.add_chip(100).unwrap();
    assert!(game.can_deal());
}

#[test]
fn next_round_clears_the_table_but_keeps_the_bankroll() {
    let script = [
        (Rank::Ace, Suit::Spades),
        (Rank::Nine, Suit::Clubs),
        (Rank::King, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
    ];
    let mut game = standard_game(&script);

    game.set_bet(1_000).unwrap();
    game.deal().unwrap();
    assert_eq!(game.bankroll(), 11_500);

    game.next_round().unwrap();

    assert_eq!(game.phase(), Phase::Betting);
    assert!(game.hands().is_empty());
    assert!(game.dealer().is_none());
    assert_eq!(game.bankroll(), 11_500);
    assert_eq!(game.pending_bet(), 0);
}

#[test]
fn free_funds_can_be_added_after_a_cold_streak() {
    let mut game = stacked_game(Rules::vegas_classic(), &[], 0);

    game.add_funds(1_000);
    game.set_bet(500).unwrap();
    assert!(game.can_deal());
}
