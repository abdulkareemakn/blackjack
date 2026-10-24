use blackjack_core::{BlackjackGame, Card, Hand, HandOutcome, Phase, Rules};
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable};
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::theme::{
    CREAM, FELT_DEEP, FELT_LIGHT, GOLD, GOLD_LIGHT, INK, card_shadow, cream, gold, gradient, loss,
    money, push, rule_line, tone, win,
};
use crate::{ClearBet, DoubleDown, Hit, Primary, Split, Stand};

const CHIPS: [(u64, u32, u32); 4] = [
    (500, 0xA93232, 0xCB4B4B),
    (2_500, 0x1B6E4A, 0x2C8C61),
    (10_000, 0x232326, 0x3D3D44),
    (50_000, 0x563A7D, 0x7050A0),
];

const DEALER_CARD_W: f32 = 82.0;
const DEALER_CARDS_MAX_W: f32 = 252.0;
const PLAYER_CARD_W: f32 = 74.0;
const PLAYER_CARDS_MAX_W: f32 = 200.0;
const TRAY_H: f32 = 150.0;

pub struct BlackjackTable {
    game: BlackjackGame,
    focus_handle: FocusHandle,
}

impl BlackjackTable {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            game: BlackjackGame::new(Rules::vegas_classic()),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn on_hit(&mut self, cx: &mut Context<Self>) {
        if self.game.can_hit() {
            let _ = self.game.hit();
            cx.notify();
        }
    }

    fn on_stand(&mut self, cx: &mut Context<Self>) {
        if self.game.can_stand() {
            let _ = self.game.stand();
            cx.notify();
        }
    }

    fn on_double(&mut self, cx: &mut Context<Self>) {
        if self.game.can_double() {
            let _ = self.game.double_down();
            cx.notify();
        }
    }

    fn on_split(&mut self, cx: &mut Context<Self>) {
        if self.game.can_split() {
            let _ = self.game.split();
            cx.notify();
        }
    }

    fn on_primary(&mut self, cx: &mut Context<Self>) {
        match self.game.phase() {
            Phase::Betting if self.game.can_deal() => {
                let _ = self.game.deal();
                cx.notify();
            }
            Phase::Settled => {
                let _ = self.game.next_round();
                cx.notify();
            }
            _ => {}
        }
    }

    fn on_clear(&mut self, cx: &mut Context<Self>) {
        if self.game.phase() == Phase::Betting && self.game.pending_bet() > 0 {
            let _ = self.game.clear_bet();
            cx.notify();
        }
    }

    fn on_chip(&mut self, amount: u64, cx: &mut Context<Self>) {
        if self.game.can_add_chip(amount) {
            let _ = self.game.add_chip(amount);
            cx.notify();
        }
    }

    fn on_add_funds(&mut self, cx: &mut Context<Self>) {
        self.game.add_funds(50_000);
        cx.notify();
    }

    // ── Layout ─────────────────────────────────────────────────────────────

    fn render_header(&self, cx: &mut Context<Self>) -> Div {
        div()
            .h(px(78.))
            .flex_none()
            .w_full()
            .px(px(30.))
            .flex()
            .items_center()
            .justify_between()
            .border_b(px(1.))
            .border_color(gold(0.16))
            .bg(rgba(0x03110CCC))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(14.))
                    .child(
                        div()
                            .size(px(42.))
                            .rounded_full()
                            .border(px(1.))
                            .border_color(gold(0.45))
                            .bg(gradient(135., gold(0.16), gold(0.03)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .font_family("Marcellus")
                                    .text_size(px(22.))
                                    .text_color(gold(0.95))
                                    .child("♠"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .font_family("Marcellus")
                                    .text_size(px(23.))
                                    .text_color(cream(0.98))
                                    .child("BLACKJACK"),
                            )
                            .child(
                                div()
                                    .text_size(px(9.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(gold(0.58))
                                    .child("VEGAS CLASSIC  ·  SIX DECKS  ·  PAYS 3 TO 2"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(18.))
                    .child(self.render_shoe())
                    .child(div().w(px(1.)).h(px(36.)).bg(gold(0.15)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_end()
                                    .gap(px(1.))
                                    .child(
                                        div()
                                            .text_size(px(9.))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(gold(0.55))
                                            .child("BANKROLL"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(22.))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(cream(0.98))
                                            .child(money(self.game.bankroll())),
                                    ),
                            )
                            .child(add_funds_button(cx)),
                    ),
            )
    }

    fn render_shoe(&self) -> Div {
        let total = self.game.shoe_total().max(1);
        let fraction = (self.game.shoe_remaining() as f32 / total as f32).clamp(0.0, 1.0);

        div()
            .w(px(96.))
            .flex()
            .flex_col()
            .gap(px(5.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_size(px(8.5))
                    .font_weight(FontWeight::BOLD)
                    .child(div().text_color(gold(0.5)).child("SHOE"))
                    .child(
                        div()
                            .text_color(gold(0.8))
                            .child(format!("{}%", (fraction * 100.0).round() as u32)),
                    ),
            )
            .child(
                div()
                    .h(px(4.))
                    .w_full()
                    .rounded_full()
                    .bg(rgba(0xFFFFFF14))
                    .child(
                        div()
                            .h_full()
                            .w(relative(fraction))
                            .rounded_full()
                            .bg(gold(0.75)),
                    ),
            )
    }

    fn render_table(&self) -> Div {
        div()
            .flex_1()
            .w_full()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .px(px(28.))
            .pt(px(12.))
            .pb(px(4.))
            .child(
                div()
                    .absolute()
                    .top(px(16.))
                    .left(px(30.))
                    .right(px(30.))
                    .bottom(px(8.))
                    .rounded(px(290.))
                    .border(px(1.))
                    .border_color(gold(0.12)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(24.))
                    .left(px(38.))
                    .right(px(38.))
                    .bottom(px(16.))
                    .rounded(px(282.))
                    .border(px(1.))
                    .border_color(gold(0.06)),
            )
            .child(
                div()
                    .absolute()
                    .top(px(18.))
                    .right(px(36.))
                    .child(self.render_shoe()),
            )
            .child(self.render_dealer())
            .child(self.render_banner())
            .child(self.render_player())
    }

    fn render_dealer(&self) -> Div {
        let dealer = self.game.dealer();
        let (label, color) = match self.game.phase() {
            Phase::Betting => ("WAITING".to_string(), gold(0.5)),
            Phase::PlayerTurn => {
                let text = dealer
                    .and_then(|hand| hand.cards().first().copied())
                    .map(|card| format!("SHOWS {}", short_card(card)))
                    .unwrap_or_else(|| "SHOWS —".to_string());
                (text, cream(0.85))
            }
            Phase::Settled => match dealer {
                Some(hand) => {
                    let value = hand.value();
                    let text = if value.is_bust() {
                        "BUST".to_string()
                    } else if hand.is_blackjack() {
                        "BLACKJACK".to_string()
                    } else if value.soft && value.total < 21 {
                        format!("SOFT {}", value.total)
                    } else {
                        value.total.to_string()
                    };
                    let color = if value.is_bust() {
                        loss(1.0)
                    } else if hand.is_blackjack() {
                        gold(1.0)
                    } else {
                        cream(0.9)
                    };
                    (text, color)
                }
                None => ("—".to_string(), cream(0.5)),
            },
        };

        let cards = match dealer {
            Some(hand) => card_row(
                hand,
                DEALER_CARD_W,
                self.game.dealer_hole_hidden(),
                DEALER_CARDS_MAX_W,
            ),
            None => div()
                .flex()
                .gap(px(10.))
                .child(empty_card(DEALER_CARD_W))
                .child(empty_card(DEALER_CARD_W)),
        };

        div()
            .flex_none()
            .h(px(188.))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(gold(0.58))
                            .child("DEALER"),
                    )
                    .child(value_pill(label, color, false)),
            )
            .child(cards)
    }

    fn render_banner(&self) -> Div {
        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(18.))
            .py(px(8.))
            .child(rule_line(64.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(2.))
                    .child(
                        div()
                            .font_family("Marcellus")
                            .text_size(px(19.))
                            .text_color(gold(0.32))
                            .child("BLACKJACK PAYS 3 TO 2"),
                    )
                    .child(
                        div()
                            .text_size(px(9.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(gold(0.2))
                            .child("DEALER MUST STAND ON SOFT 17"),
                    ),
            )
            .child(rule_line(64.).flex_row_reverse())
    }

    fn render_player(&self) -> Div {
        let mut area = div()
            .flex_1()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(12.))
            .pt(px(2.));

        if self.game.phase() == Phase::Betting {
            area = area.child(self.render_betting_spot());
        } else {
            for (index, hand) in self.game.hands().iter().enumerate() {
                area = area.child(self.render_player_hand(hand, index));
            }
        }

        area
    }

    fn render_betting_spot(&self) -> Div {
        let bet = self.game.pending_bet();
        let broke = self.game.bankroll() < self.game.rules().min_bet;

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(11.))
            .child(
                div()
                    .w(px(176.))
                    .h(px(176.))
                    .rounded_full()
                    .border(px(2.))
                    .border_dashed()
                    .border_color(gold(0.3))
                    .bg(rgba(0x0000001A))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(3.))
                    .child(
                        div()
                            .text_size(px(9.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(gold(0.5))
                            .child("YOUR BET"),
                    )
                    .child(
                        div()
                            .text_size(px(if bet == 0 { 28. } else { 34. }))
                            .font_family("Marcellus")
                            .text_color(if bet == 0 {
                                cream(0.32)
                            } else {
                                cream(0.98)
                            })
                            .child(if bet == 0 {
                                "$0".to_string()
                            } else {
                                money(bet)
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(8.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(gold(0.38))
                            .child("MIN $5  ·  MAX $500"),
                    ),
            )
            .child(
                div()
                    .text_size(px(9.5))
                    .text_color(if broke { loss(0.9) } else { gold(0.42) })
                    .child(if broke {
                        "BANKROLL BELOW THE TABLE MINIMUM — USE ADD $500".to_string()
                    } else {
                        "SELECT CHIPS BELOW TO BUILD YOUR WAGER".to_string()
                    }),
            )
    }

    fn render_player_hand(&self, hand: &Hand, index: usize) -> Div {
        let active =
            self.game.phase() == Phase::PlayerTurn && index == self.game.active_hand_index();
        let multiple = self.game.hands().len() > 1;
        let value = hand.value();
        let value_color = if value.is_bust() {
            loss(0.95)
        } else if hand.is_blackjack() {
            gold(1.0)
        } else {
            cream(0.92)
        };

        let outcome_slot: AnyElement = match hand.outcome() {
            Some(outcome) => {
                let (text, color) = match outcome {
                    HandOutcome::Blackjack => (
                        format!(
                            "BLACKJACK  +{}",
                            money(hand.payout().saturating_sub(hand.bet()))
                        ),
                        gold(1.0),
                    ),
                    HandOutcome::Win => (format!("WIN  +{}", money(hand.bet())), win(1.0)),
                    HandOutcome::Push => ("PUSH".to_string(), push(1.0)),
                    HandOutcome::Loss => (format!("LOSE  -{}", money(hand.bet())), loss(1.0)),
                };

                div()
                    .h(px(20.))
                    .flex()
                    .items_center()
                    .child(mini_badge(text, color))
                    .into_any_element()
            }
            None => div().h(px(20.)).into_any_element(),
        };

        div()
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(7.))
            .p(px(10.))
            .rounded(px(18.))
            .border(px(1.))
            .border_color(if active { gold(0.55) } else { gold(0.0) })
            .bg(if active {
                gold(0.05)
            } else {
                rgba(0x00000000)
            })
            .child(card_row(
                hand,
                PLAYER_CARD_W,
                false,
                PLAYER_CARDS_MAX_W,
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .when(multiple, |row| {
                        row.child(
                            div()
                                .text_size(px(8.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(gold(0.5))
                                .child(format!("H{}", index + 1)),
                        )
                    })
                    .child(value_pill(value_text(hand), value_color, active))
                    .child(
                        div()
                            .text_size(px(9.))
                            .text_color(gold(0.6))
                            .child(format!("WAGER {}", money(hand.bet()))),
                    ),
            )
            .child(outcome_slot)
    }

    // ── Control tray ───────────────────────────────────────────────────────

    fn render_tray(&self, cx: &mut Context<Self>) -> Div {
        match self.game.phase() {
            Phase::Betting => self.render_bet_tray(cx),
            Phase::PlayerTurn => self.render_turn_tray(cx),
            Phase::Settled => self.render_settled_tray(cx),
        }
    }

    fn render_bet_tray(&self, cx: &mut Context<Self>) -> Div {
        let bet = self.game.pending_bet();
        let broke = self.game.bankroll() < self.game.rules().min_bet;
        let can_deal = self.game.can_deal();

        let mut chip_row = div().flex().items_center().gap(px(12.));
        for (amount, base, light) in CHIPS {
            chip_row = chip_row.child(self.render_chip(amount, base, light, cx));
        }

        tray_shell()
            .child(
                div()
                    .flex_none()
                    .min_w(px(230.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(small_label("PLACE YOUR BET"))
                    .child(
                        div()
                            .text_size(px(30.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(if bet == 0 { cream(0.32) } else { cream(0.98) })
                            .child(if bet == 0 {
                                "$0".to_string()
                            } else {
                                money(bet)
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(9.5))
                            .text_color(if broke { loss(0.9) } else { gold(0.45) })
                            .child(format!(
                                "BANKROLL {}  ·  MIN $5  ·  MAX $500",
                                money(self.game.bankroll())
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(9.))
                    .child(chip_row)
                    .child(hint("CLICK A CHIP TO ADD TO YOUR WAGER  ·  N / SPACE DEALS")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(action_button(
                        "clear-bet",
                        "CLEAR",
                        bet > 0,
                        0xFFFFFF14,
                        CREAM,
                        0xFFFFFF26,
                        96.,
                        cx,
                        |this, cx| this.on_clear(cx),
                    ))
                    .child(action_button(
                        "deal",
                        "DEAL",
                        can_deal,
                        GOLD,
                        INK,
                        GOLD_LIGHT,
                        148.,
                        cx,
                        |this, cx| this.on_primary(cx),
                    )),
            )
    }

    fn render_chip(&self, amount: u64, base: u32, light: u32, cx: &mut Context<Self>) -> Stateful<Div> {
        let enabled = self.game.can_add_chip(amount);

        let mut chip = div()
            .id(format!("chip-{amount}"))
            .flex_none()
            .size(px(62.))
            .rounded_full()
            .bg(gradient(135., rgb(light), rgb(base)))
            .border(px(2.))
            .border_color(rgba(0xFFFFFF3D))
            .shadow(vec![
                BoxShadow::new(px(0.), px(4.), hsla(0., 0., 0., 0.35)).blur_radius(px(8.)),
            ])
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .size(px(48.))
                    .rounded_full()
                    .border(px(1.))
                    .border_dashed()
                    .border_color(rgba(0xFFFFFF8A))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(cream(0.98))
                            .child(money(amount)),
                    ),
            );

        if enabled {
            chip = chip
                .cursor_pointer()
                .hover(|style| style.border_color(rgba(0xFFFFFFB8)))
                .active(|style| style.opacity(0.82))
                .on_click(
                    cx.listener(move |this, _: &ClickEvent, _, cx| this.on_chip(amount, cx)),
                );
        } else {
            chip = chip.opacity(0.32);
        }

        chip
    }

    fn render_turn_tray(&self, cx: &mut Context<Self>) -> Div {
        let active_value = self
            .game
            .active_hand()
            .map(value_text)
            .unwrap_or_else(|| "—".to_string());
        let hand_count = self.game.hands().len();
        let active_number = self.game.active_hand_index() + 1;
        let turn_label = if hand_count > 1 {
            format!("YOUR TURN  ·  HAND {active_number} OF {hand_count}")
        } else {
            "YOUR TURN".to_string()
        };

        tray_shell()
            .child(
                div()
                    .flex_none()
                    .min_w(px(230.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(small_label(&turn_label))
                    .child(
                        div()
                            .text_size(px(30.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(cream(0.98))
                            .child(active_value),
                    )
                    .child(
                        div()
                            .text_size(px(9.5))
                            .text_color(gold(0.45))
                            .child(format!(
                                "BANKROLL {}  ·  IN PLAY {}",
                                money(self.game.bankroll()),
                                money(self.game.total_in_play())
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(action_button(
                                "hit",
                                "HIT",
                                self.game.can_hit(),
                                CREAM,
                                INK,
                                0xFFF8EA,
                                104.,
                                cx,
                                |this, cx| this.on_hit(cx),
                            ))
                            .child(action_button(
                                "stand",
                                "STAND",
                                self.game.can_stand(),
                                0x1E5A43,
                                CREAM,
                                0x277055,
                                104.,
                                cx,
                                |this, cx| this.on_stand(cx),
                            ))
                            .child(action_button(
                                "double",
                                "DOUBLE",
                                self.game.can_double(),
                                0xC9A24B,
                                INK,
                                0xE0BC6B,
                                104.,
                                cx,
                                |this, cx| this.on_double(cx),
                            ))
                            .child(action_button(
                                "split",
                                "SPLIT",
                                self.game.can_split(),
                                0x5E3F87,
                                CREAM,
                                0x74519F,
                                104.,
                                cx,
                                |this, cx| this.on_split(cx),
                            )),
                    )
                    .child(hint(
                        "HIT  H   ·   STAND  S   ·   DOUBLE  D   ·   SPLIT  P",
                    )),
            )
            .child(self.render_stats())
    }

    fn render_stats(&self) -> Div {
        div()
            .flex_none()
            .min_w(px(150.))
            .flex()
            .flex_col()
            .items_end()
            .gap(px(3.))
            .child(small_label("IN PLAY"))
            .child(
                div()
                    .text_size(px(19.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(cream(0.96))
                    .child(money(self.game.total_in_play())),
            )
            .child(
                div()
                    .text_size(px(9.5))
                    .text_color(gold(0.5))
                    .child(format!("BANKROLL {}", money(self.game.bankroll()))),
            )
    }

    fn render_settled_tray(&self, cx: &mut Context<Self>) -> Div {
        let net = self.game.net_result();
        let (headline, color) = if net > 0 {
            ("YOU WIN", win(1.0))
        } else if net < 0 {
            ("YOU LOSE", loss(1.0))
        } else {
            ("PUSH", push(1.0))
        };
        let amount = if net > 0 {
            format!("  +{}", money(net as u64))
        } else if net < 0 {
            format!("  -{}", money((-net) as u64))
        } else {
            String::new()
        };

        let multiple = self.game.hands().len() > 1;
        let mut summary = div().flex().items_center().gap(px(8.)).flex_wrap();
        for (index, hand) in self.game.hands().iter().enumerate() {
            if let Some(outcome) = hand.outcome() {
                let (label, badge_color) = match outcome {
                    HandOutcome::Blackjack => (
                        format!(
                            "BJ +{}",
                            money(hand.payout().saturating_sub(hand.bet()))
                        ),
                        gold(1.0),
                    ),
                    HandOutcome::Win => (format!("WIN +{}", money(hand.bet())), win(1.0)),
                    HandOutcome::Push => ("PUSH".to_string(), push(1.0)),
                    HandOutcome::Loss => (format!("LOSE -{}", money(hand.bet())), loss(1.0)),
                };
                let prefix = if multiple {
                    format!("H{} ", index + 1)
                } else {
                    String::new()
                };
                summary = summary.child(mini_badge(format!("{prefix}{label}"), badge_color));
            }
        }

        tray_shell()
            .child(
                div()
                    .flex_none()
                    .min_w(px(270.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(small_label("ROUND COMPLETE"))
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(color)
                            .child(format!("{headline}{amount}")),
                    )
                    .child(
                        div()
                            .text_size(px(9.5))
                            .text_color(gold(0.45))
                            .child(self.settled_subtitle()),
                    ),
            )
            .child(summary)
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap(px(6.))
                    .child(action_button(
                        "next-hand",
                        "NEXT HAND",
                        true,
                        GOLD,
                        INK,
                        GOLD_LIGHT,
                        148.,
                        cx,
                        |this, cx| this.on_primary(cx),
                    ))
                    .child(hint("SPACE / N DEALS AGAIN")),
            )
    }

    fn settled_subtitle(&self) -> String {
        let dealer_text = match self.game.dealer() {
            Some(hand) => {
                let value = hand.value();
                if value.is_bust() {
                    "Dealer bust".to_string()
                } else if hand.is_blackjack() {
                    "Dealer blackjack".to_string()
                } else {
                    format!("Dealer {}", value.total)
                }
            }
            None => "Dealer —".to_string(),
        };
        let hand_count = self.game.hands().len();
        let hands_text = if hand_count == 1 {
            "1 hand".to_string()
        } else {
            format!("{hand_count} hands")
        };

        format!(
            "{}  ·  {}  ·  Bankroll {}",
            dealer_text,
            hands_text,
            money(self.game.bankroll())
        )
    }
}

impl Render for BlackjackTable {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("blackjack-table")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Hit, _, cx| this.on_hit(cx)))
            .on_action(cx.listener(|this, _: &Stand, _, cx| this.on_stand(cx)))
            .on_action(cx.listener(|this, _: &DoubleDown, _, cx| this.on_double(cx)))
            .on_action(cx.listener(|this, _: &Split, _, cx| this.on_split(cx)))
            .on_action(cx.listener(|this, _: &Primary, _, cx| this.on_primary(cx)))
            .on_action(cx.listener(|this, _: &ClearBet, _, cx| this.on_clear(cx)))
            .size_full()
            .flex()
            .flex_col()
            .font_family("Lato")
            .text_color(cream(0.95))
            .bg(gradient(165., tone(FELT_LIGHT, 1.0), tone(FELT_DEEP, 1.0)))
            .child(self.render_header(cx))
            .child(self.render_table())
            .child(self.render_tray(cx))
    }
}

fn add_funds_button(cx: &mut Context<BlackjackTable>) -> Stateful<Div> {
    div()
        .id("add-funds")
        .px(px(10.))
        .py(px(6.))
        .rounded_full()
        .border(px(1.))
        .border_color(gold(0.25))
        .text_size(px(9.))
        .font_weight(FontWeight::BOLD)
        .text_color(gold(0.85))
        .cursor_pointer()
        .hover(|style| style.bg(gold(0.1)).border_color(gold(0.5)))
        .child("ADD $500")
        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.on_add_funds(cx)))
}

#[allow(clippy::too_many_arguments)]
fn action_button(
    id: &'static str,
    label: &'static str,
    enabled: bool,
    fill: u32,
    foreground: u32,
    hover: u32,
    min_width: f32,
    cx: &mut Context<BlackjackTable>,
    on_click: impl Fn(&mut BlackjackTable, &mut Context<BlackjackTable>) + 'static,
) -> Button {
    Button::new(id)
        .label(label)
        .large()
        .min_w(px(min_width))
        .disabled(!enabled)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(rgb(fill).into())
                .foreground(rgb(foreground).into())
                .hover(rgb(hover).into())
                .active(rgb(fill).into())
                .shadow(true),
        )
        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| on_click(this, cx)))
}

fn tray_shell() -> Div {
    div()
        .h(px(TRAY_H))
        .flex_none()
        .w_full()
        .px(px(34.))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(22.))
        .border_t(px(1.))
        .border_color(gold(0.16))
        .bg(rgba(0x02100AD9))
}

fn small_label(text: &str) -> Div {
    div()
        .text_size(px(9.5))
        .font_weight(FontWeight::BOLD)
        .text_color(gold(0.6))
        .child(text.to_string())
}

fn hint(text: &str) -> Div {
    div()
        .text_size(px(9.))
        .text_color(gold(0.38))
        .child(text.to_string())
}

fn mini_badge(text: String, color: Rgba) -> Div {
    div()
        .px(px(8.))
        .py(px(2.))
        .rounded_full()
        .bg(color.opacity(0.12))
        .border(px(1.))
        .border_color(color.opacity(0.32))
        .text_size(px(9.))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .child(text)
}

fn value_pill(text: String, color: Rgba, active: bool) -> Div {
    div()
        .px(px(10.))
        .py(px(3.))
        .rounded_full()
        .border(px(1.))
        .border_color(color.opacity(if active { 0.55 } else { 0.26 }))
        .bg(color.opacity(if active { 0.16 } else { 0.07 }))
        .text_size(px(11.))
        .font_weight(FontWeight::BOLD)
        .text_color(color)
        .child(text)
}

fn value_text(hand: &Hand) -> String {
    let value = hand.value();

    if value.is_bust() {
        "BUST".to_string()
    } else if hand.is_blackjack() {
        "BLACKJACK".to_string()
    } else if value.soft && value.total < 21 {
        format!("SOFT {}", value.total)
    } else {
        value.total.to_string()
    }
}

fn short_card(card: Card) -> String {
    if card.is_ace() {
        "A".to_string()
    } else {
        card.value().to_string()
    }
}

fn card_row(hand: &Hand, width: f32, hide_hole: bool, max_width: f32) -> Div {
    let count = hand.cards().len();
    let desired_step = width * 0.38;
    let step = if count > 1 {
        desired_step.min(((max_width - width).max(0.0)) / (count - 1) as f32)
    } else {
        0.0
    };
    let row_width = width + step * count.saturating_sub(1) as f32;

    let mut row = div()
        .relative()
        .flex_none()
        .w(px(row_width))
        .h(px(width * 1.4));

    for (index, card) in hand.cards().iter().enumerate() {
        row = row.child(
            card_element(*card, width, hide_hole && index == 1)
                .absolute()
                .top(px(0.))
                .left(px(index as f32 * step)),
        );
    }

    row
}

fn card_element(card: Card, width: f32, face_down: bool) -> Div {
    let path = if face_down {
        "cards/back.svg".to_string()
    } else {
        format!("cards/{}{}.svg", card.rank.label(), card.suit.code())
    };

    div()
        .w(px(width))
        .h(px(width * 1.4))
        .rounded(px(width * 0.075))
        .overflow_hidden()
        .bg(rgb(0xF8F1E1))
        .shadow(card_shadow())
        .child(img(path).size_full())
}

fn empty_card(width: f32) -> Div {
    div()
        .w(px(width))
        .h(px(width * 1.4))
        .rounded(px(width * 0.075))
        .border(px(1.))
        .border_dashed()
        .border_color(gold(0.18))
        .bg(rgba(0x00000018))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .font_family("Marcellus")
                .text_size(px(width * 0.34))
                .text_color(gold(0.14))
                .child("♠"),
        )
}
