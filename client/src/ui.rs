//! Handles User Interface interactions such as text and buttons.

use macroquad::{
    color::Color,
    math::Vec2,
    text::draw_text,
    ui::{
        hash, root_ui,
        widgets::{Label, Slider},
        Id, Skin, Style, Ui,
    },
};
use netcode::client::Game;

/// Draw the GUI to the canvas
pub fn draw_ui(
    game: &mut Game,
    label_skin: &Skin,
    active_skin: &Skin,
    inactive_skin: &Skin,
) {
    let Game {
        interpolation,
        reconciliation,
        prediction,
        ..
    } = game;

    root_ui().pop_skin();
    root_ui().push_skin(&label_skin);

    Label::new(format!("Ping {:?}ms", game.ping_cache))
        .position(Vec2 { x: 15., y: 15. })
        .ui(&mut root_ui());

    change_style(*prediction, active_skin, inactive_skin);
    Label::new(format!("Prediction"))
        .position(Vec2 { x: 150., y: 15. })
        .ui(&mut root_ui());

    change_style(*reconciliation, active_skin, inactive_skin);
    Label::new(format!("Reconciliation"))
        .position(Vec2 { x: 300., y: 15. })
        .ui(&mut root_ui());

    change_style(*interpolation, active_skin, inactive_skin);
    Label::new(format!("Interpolation"))
        .position(Vec2 { x: 500., y: 15. })
        .ui(&mut root_ui());
}

/// Changes the game's skin to apply some styling based on a condition.
/// Removes the current skin before pushing the replacement.
fn change_style(condition: bool, active_skin: &Skin, inactive_skin: &Skin) {
    root_ui().pop_skin();

    let skin = if condition {
        active_skin
    } else {
        inactive_skin
    };

    root_ui().push_skin(&skin);
}
