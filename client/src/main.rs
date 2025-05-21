use std::{collections::HashSet, time::Duration};

use chrono::Utc;
use macroquad::{
    color::{BLUE, GREEN},
    input::{get_keys_down, get_keys_pressed, KeyCode},
    shapes::draw_rectangle,
    window::{next_frame, screen_height, screen_width},
};
use netcode::{client::Game, State};
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;

const PLAYER_SIZE: f32 = 30.;
const PLAYER_SPEED: f32 = 3.;
const GROUND_HEIGHT: f32 = 0.8;
const JUMP_MULTIPLIER: f32 = 300.;

#[macroquad::main("BasicShapes")]
async fn main() -> anyhow::Result<()> {
    let mut game = Game::new();

    loop {
        draw_ground();

        draw_players(&mut game.state);

        handle_keys(&mut game);

        game.update();

        next_frame().await;
    }
}

fn handle_key_press(key_codes: HashSet<KeyCode>, game: &mut Game) {
    for key in key_codes {
        match key {
            KeyCode::W => match game.player_id {
                Some(id) => {
                    if let Some(player) = game.state.players.iter().find(|p| p.id == id) {
                        if player.y() <= 0. {
                            game.jump();
                        }
                    }
                }
                None => {}
            },
            KeyCode::J => {
                game.join();
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
            KeyCode::D => game.move_player(PLAYER_SPEED as f32),
            KeyCode::A => game.move_player(-PLAYER_SPEED as f32),
            _ => {}
        }
    }
}

fn draw_players(state: &mut State) {
    for player in &state.players {
        draw_rectangle(
            player.x as _,
            (screen_height() * GROUND_HEIGHT) - PLAYER_SIZE - (player.y() as f32 * JUMP_MULTIPLIER),
            PLAYER_SIZE,
            PLAYER_SIZE,
            GREEN,
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
