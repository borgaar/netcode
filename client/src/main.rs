use std::{collections::HashSet, ops::Not};

use macroquad::{
    color::{Color, BLUE, GREEN, PURPLE, RED, YELLOW},
    input::{get_keys_down, get_keys_pressed, KeyCode},
    shapes::draw_rectangle,
    time::get_frame_time,
    ui::{root_ui, Skin},
    window::{next_frame, screen_height, screen_width},
};
use netcode::{client::Game, State, MAX_UNITS_PER_SECOND};
use ui::draw_ui;

const PLAYER_SIZE: f32 = 30.;
const GROUND_HEIGHT: f32 = 0.8;
const JUMP_MULTIPLIER: f32 = 300.;
const PLAYER_COLORS: [Color; 5] = [RED, GREEN, BLUE, YELLOW, PURPLE];
const PIXELS_PER_UNIT: f32 = 40.;

mod ui;

#[macroquad::main("BasicShapes")]
async fn main() -> anyhow::Result<()> {
    let mut game = Game::new();

    let font = include_bytes!("./font.ttf");

    let label_style = root_ui()
        .style_builder()
        .font(font)
        .unwrap()
        .text_color(Color::from_hex(0xFFFFFF))
        .font_size(24)
        .build();

    let inactive_style = root_ui()
        .style_builder()
        .font(font)
        .unwrap()
        .text_color(Color::from_hex(0xFF7B9C))
        .font_size(24)
        .build();

    let active_style = root_ui()
        .style_builder()
        .font(font)
        .unwrap()
        .text_color(Color::from_hex(0x6A994E))
        .font_size(24)
        .build();

    let label_skin = {
        Skin {
            label_style,
            ..root_ui().default_skin()
        }
    };

    let active_skin = {
        Skin {
            label_style: active_style,
            ..root_ui().default_skin()
        }
    };

    let inactive_skin = {
        Skin {
            label_style: inactive_style,
            ..root_ui().default_skin()
        }
    };

    loop {
        draw_ground();

        draw_players(&mut game.display_state);

        handle_keys(&mut game);

        draw_ui(&mut game, &label_skin, &active_skin, &inactive_skin);

        game.update();

        next_frame().await;
    }
}

fn handle_key_press(key_codes: HashSet<KeyCode>, game: &mut Game) {
    for key in key_codes {
        match key {
            KeyCode::W => match game.player_idx {
                Some(idx) => {
                    if let Some(player) = game.display_state.players.get(&idx) {
                        if player.y() <= 0. {
                            game.jump();
                        }
                    }
                }
                None => {}
            },
            KeyCode::Space => {
                if let None = game.player_idx {
                    game.join();
                }
            }
            KeyCode::J => {
                let new_ping = game.ping_cache.checked_sub(10).unwrap_or(0);
                game.set_simulated_ping(new_ping);
            }
            KeyCode::K => {
                let new_ping = game.ping_cache + 10;
                game.set_simulated_ping(new_ping);
            }
            KeyCode::P => {
                game.prediction = !game.prediction;
                if !game.prediction {
                    game.reconciliation = false;
                }
            }
            KeyCode::I => {
                game.interpolation = !game.interpolation;
            }
            KeyCode::R => {
                game.reconciliation = !game.reconciliation;
                if !game.prediction {
                    game.prediction = true
                }
            }
            _ => {}
        }
    }
}

fn handle_keys(game: &mut Game) {
    let keys_down = get_keys_down();
    handle_key_hold(keys_down, game);

    let keys_pressed = get_keys_pressed();
    handle_key_press(keys_pressed, game);
}

fn handle_key_hold(key_codes: HashSet<KeyCode>, game: &mut Game) {
    for key in key_codes {
        match key {
            KeyCode::D => game.move_player(MAX_UNITS_PER_SECOND as f32 * get_frame_time()),
            KeyCode::A => game.move_player(-MAX_UNITS_PER_SECOND as f32 * get_frame_time()),
            _ => {}
        }
    }
}

fn draw_players(state: &mut State) {
    for (idx, player) in state.players.iter() {
        draw_rectangle(
            player.x as f32 * PIXELS_PER_UNIT,
            (screen_height() * GROUND_HEIGHT) - PLAYER_SIZE - (player.y() as f32 * JUMP_MULTIPLIER),
            PLAYER_SIZE,
            PLAYER_SIZE,
            PLAYER_COLORS[idx % PLAYER_COLORS.len()],
        );
    }
}

fn draw_ground() {
    draw_rectangle(
        0.,
        screen_height() * GROUND_HEIGHT,
        screen_width(),
        screen_height(),
        BLUE,
    );
}
