use gpui_kit::*;

pub const GOLD: u32 = 0xD4B36A;
pub const GOLD_LIGHT: u32 = 0xE8D196;
pub const GOLD_DEEP: u32 = 0xA9873F;
pub const CREAM: u32 = 0xF6EED9;
pub const INK: u32 = 0x0B1F17;
pub const FELT: u32 = 0x0B2E22;
pub const FELT_LIGHT: u32 = 0x123D2D;
pub const FELT_DEEP: u32 = 0x04120D;
pub const WIN: u32 = 0x63B98A;
pub const LOSS: u32 = 0xC05A56;
pub const PUSH: u32 = 0xCFC3A6;

pub fn tone(hex: u32, alpha: f32) -> Rgba {
    rgb(hex).opacity(alpha)
}

pub fn gold(alpha: f32) -> Rgba {
    tone(GOLD, alpha)
}

pub fn cream(alpha: f32) -> Rgba {
    tone(CREAM, alpha)
}

pub fn win(alpha: f32) -> Rgba {
    tone(WIN, alpha)
}

pub fn loss(alpha: f32) -> Rgba {
    tone(LOSS, alpha)
}

pub fn push(alpha: f32) -> Rgba {
    tone(PUSH, alpha)
}

pub fn gradient(angle: f32, from: Rgba, to: Rgba) -> Background {
    linear_gradient(
        angle,
        linear_color_stop(from, 0.0),
        linear_color_stop(to, 1.0),
    )
}

/// Formats a cent amount as a dollar string with grouping, e.g. `$1,250`.
pub fn money(cents: u64) -> String {
    let dollars = cents / 100;
    let remainder = cents % 100;
    let digits = dollars.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }

    if remainder == 0 {
        format!("${grouped}")
    } else {
        format!("${grouped}.{remainder:02}")
    }
}

pub fn card_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow::new(px(0.), px(8.), hsla(155., 0.42, 0.02, 0.48)).blur_radius(px(14.)),
        BoxShadow::new(px(0.), px(1.), hsla(0., 0., 0., 0.28)).blur_radius(px(2.)),
    ]
}

pub fn rule_line(width: f32) -> Div {
    div()
        .w(px(width))
        .h(px(1.))
        .flex_none()
        .bg(gradient(90., gold(0.0), gold(0.22)))
}
