mod assets;
mod table;
mod theme;

use std::borrow::Cow;

use assets::Assets;
use gpui_kit::*;
use table::BlackjackTable;

actions!(
    blackjack,
    [Hit, Stand, DoubleDown, Split, Primary, ClearBet, Quit]
);

fn load_fonts(cx: &App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Lato-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Lato-Bold.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Marcellus-Regular.ttf").as_slice()),
    ];

    cx.text_system()
        .add_fonts(fonts)
        .expect("bundled fonts are valid");
}

fn main() {
    gpui_kit::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            gpui_kit::init(cx);
            load_fonts(cx);

            cx.bind_keys([
                KeyBinding::new("h", Hit, None),
                KeyBinding::new("s", Stand, None),
                KeyBinding::new("d", DoubleDown, None),
                KeyBinding::new("p", Split, None),
                KeyBinding::new("n", Primary, None),
                KeyBinding::new("space", Primary, None),
                KeyBinding::new("escape", ClearBet, None),
                KeyBinding::new("q", Quit, None),
            ]);

            cx.on_action(|_: &Quit, cx| cx.quit());

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1180.), px(800.)),
                    cx,
                ))),
                window_min_size: Some(size(px(980.), px(720.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Blackjack".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                app_id: Some("blackjack".to_string()),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                let table = cx.new(|cx| BlackjackTable::new(cx));
                let focus = table.read(cx).focus_handle().clone();
                window.focus(&focus, cx);
                table
            })
            .expect("failed to open the blackjack table window");

            cx.activate(true);
        });
}
