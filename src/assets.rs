use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

macro_rules! embedded_cards {
    ($($file:literal),* $(,)?) => {
        const CARD_ASSETS: &[(&str, &[u8])] = &[
            $(
                (
                    concat!("cards/", $file),
                    include_bytes!(concat!("../assets/cards/", $file)),
                ),
            )*
        ];
    };
}

embedded_cards![
    "10C.svg", "10D.svg", "10H.svg", "10S.svg", "2C.svg", "2D.svg", "2H.svg", "2S.svg",
    "3C.svg", "3D.svg", "3H.svg", "3S.svg", "4C.svg", "4D.svg", "4H.svg", "4S.svg", "5C.svg",
    "5D.svg", "5H.svg", "5S.svg", "6C.svg", "6D.svg", "6H.svg", "6S.svg", "7C.svg", "7D.svg",
    "7H.svg", "7S.svg", "8C.svg", "8D.svg", "8H.svg", "8S.svg", "9C.svg", "9D.svg", "9H.svg",
    "9S.svg", "AC.svg", "AD.svg", "AH.svg", "AS.svg", "JC.svg", "JD.svg", "JH.svg", "JS.svg",
    "KC.svg", "KD.svg", "KH.svg", "KS.svg", "QC.svg", "QD.svg", "QH.svg", "QS.svg", "back.svg",
];

/// Bundles the card deck and falls back to the GPUI Kit icon set.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, data)) = CARD_ASSETS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(*data)));
        }

        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut names: Vec<SharedString> = CARD_ASSETS
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| SharedString::from(*name))
            .collect();

        if let Ok(icons) = gpui_kit::assets::Assets.list(path) {
            for icon in icons {
                if !names.contains(&icon) {
                    names.push(icon);
                }
            }
        }

        Ok(names)
    }
}
