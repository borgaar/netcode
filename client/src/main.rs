//! Main entrypoint for the client-side graphical application.

use std::collections::HashSet;

use macroquad::{
    audio::Sound,
    color::{Color, BLUE, GREEN, PURPLE, RED, YELLOW},
    input::{get_keys_down, get_keys_pressed, KeyCode},
    shapes::draw_rectangle,
    time::get_frame_time,
    ui::{root_ui, Skin},
    window::{next_frame, screen_height, screen_width},
};
use netcode::{client::Game, MAX_UNITS_PER_SECOND};
use ui::draw_ui;

/// Player's dimentions in x and y axis measured in pixels
const PLAYER_SIZE: f32 = 30.;

/// Height of the ground on screen
const GROUND_HEIGHT: f32 = 0.8;

/// Multiplier for jump velocity
const JUMP_MULTIPLIER: f32 = 300.;

/// All possible colors for players. Cycled through as more players join in.
const PLAYER_COLORS: [Color; 5] = [RED, GREEN, BLUE, YELLOW, PURPLE];

/// Number of pixels per server-side units. Used for rendering.
const PIXELS_PER_UNIT: f32 = 40.;

mod ui;

#[macroquad::main("BasicShapes")]
async fn main() -> anyhow::Result<()> {
    let mut game = Game::new();

    let font = include_bytes!("../assets/font.ttf");
    let join_sound = macroquad::audio::load_sound_from_bytes(include_bytes!("../assets/join.wav"))
        .await
        .unwrap();
    let ping_sound =
        macroquad::audio::load_sound_from_bytes(include_bytes!("../assets/ping_pong.wav"))
            .await
            .unwrap();

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

        draw_players(&mut game);

        handle_keys(&mut game, &join_sound, &ping_sound);

        draw_ui(&mut game, &label_skin, &active_skin, &inactive_skin);

        game.update();

        next_frame().await;
    }
}

fn handle_key_press(key_codes: HashSet<KeyCode>, game: &mut Game, join_sound: &Sound) {
    for key in key_codes {
        match key {
            KeyCode::W => match game.player_idx {
                Some(idx) => {
                    if let Some(player) = game.local_state.players.get(&idx) {
                        if player.y() <= 0. {
                            game.jump();
                        }
                    }
                }
                None => {}
            },
            KeyCode::Space => {
                if let None = game.player_idx {
                    macroquad::audio::play_sound_once(&join_sound);
                    game.join();
                }
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

fn handle_keys(game: &mut Game, join_sound: &Sound, ping_sound: &Sound) {
    let keys_down = get_keys_down();
    handle_key_hold(keys_down, game, ping_sound);

    let keys_pressed = get_keys_pressed();
    handle_key_press(keys_pressed, game, join_sound);
}

fn handle_key_hold(key_codes: HashSet<KeyCode>, game: &mut Game, ping_sound: &Sound) {
    for key in key_codes {
        match key {
            KeyCode::D => game.move_player(MAX_UNITS_PER_SECOND as f32 * get_frame_time()),
            KeyCode::A => game.move_player(-MAX_UNITS_PER_SECOND as f32 * get_frame_time()),
            KeyCode::J => {
                let new_ping = game.ping_cache.checked_sub(10).unwrap_or(0);
                game.set_simulated_ping(new_ping);
                macroquad::audio::play_sound_once(ping_sound);
            }
            KeyCode::K => {
                let new_ping = game.ping_cache + 10;
                game.set_simulated_ping(new_ping);
                macroquad::audio::play_sound_once(ping_sound);
            }
            _ => {}
        }
    }
}

fn draw_players(state: &mut Game) {
    for player in state.display_state.players.values() {
        draw_rectangle(
            player.x as f32 * PIXELS_PER_UNIT,
            (screen_height() * GROUND_HEIGHT) - PLAYER_SIZE - (player.y() as f32 * JUMP_MULTIPLIER),
            PLAYER_SIZE,
            PLAYER_SIZE,
            PLAYER_COLORS[player.id % PLAYER_COLORS.len()],
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
