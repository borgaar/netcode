use std::{collections::HashSet, time::Duration};

use chrono::Utc;
use macroquad::{
    color::{BLUE, GREEN},
    input::{get_keys_down, get_keys_pressed, KeyCode},
    shapes::draw_rectangle,
    window::{next_frame, screen_height, screen_width},
};
use netcode::{client::NetcodeClient, Player, State};
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;

const PLAYER_SIZE: f32 = 30.;
const PLAYER_SPEED: f32 = 3.;
const GROUND_HEIGHT: f32 = 0.8;
const JUMP_MULTIPLIER: f32 = 300.;

#[macroquad::main("BasicShapes")]
async fn main() -> anyhow::Result<()> {
    let mut state = State {
        players: vec![Player {
            id: 0,
            x: 0.0,
            last_jump_at: Utc::now(),
        }],
    };

    let client = NetcodeClient::new([0, 0, 0, 0], 7878);

    loop {
        draw_ground();

        draw_players(&mut state);

        handle_keys(&mut state);

        next_frame().await
    }
}

fn handle_key_press(key_codes: HashSet<KeyCode>, state: &mut State) {
    for key in key_codes {
        match key {
            KeyCode::W => {
                if state.players[0].y() == 0. {
                    state.players[0].last_jump_at = Utc::now();
                }
            }
            _ => {}
        }
    }
}

fn handle_keys(state: &mut State) {
    let keys_down = get_keys_down();
    handle_key_hold(keys_down, state);

    let keys_pressed = get_keys_pressed();
    handle_key_press(keys_pressed, state);
}

fn handle_key_hold(key_codes: HashSet<KeyCode>, state: &mut State) {
    for key in key_codes {
        match key {
            KeyCode::D => state.players[0].x += PLAYER_SPEED as f64,
            KeyCode::A => state.players[0].x -= PLAYER_SPEED as f64,
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
