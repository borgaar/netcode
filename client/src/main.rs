use std::collections::HashSet;

use macroquad::{
    color::{Color, BLUE, GREEN, PURPLE, RED, YELLOW},
    input::{get_keys_down, get_keys_pressed, KeyCode},
    shapes::draw_rectangle,
    window::{next_frame, screen_height, screen_width},
};
use netcode::{client::Game, State, MAX_UNITS_PER_MS};

const PLAYER_SIZE: f32 = 30.;
const SPEED_MULTIPLIER: f32 = 1000.;
const GROUND_HEIGHT: f32 = 0.8;
const JUMP_MULTIPLIER: f32 = 300.;
const PLAYER_COLORS: [Color; 5] = [RED, GREEN, BLUE, YELLOW, PURPLE];

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
            KeyCode::W => match game.player_idx {
                Some(idx) => {
                    if let Some(player) = game.state.players.get(idx) {
                        if player.y() <= 0. {
                            game.jump();
                        }
                    }
                }
                None => {}
            },
            KeyCode::J => {
                if let None = game.player_idx {
                    game.join();
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
            KeyCode::D => game.move_player(MAX_UNITS_PER_MS as f32),
            KeyCode::A => game.move_player(-MAX_UNITS_PER_MS as f32),
            _ => {}
        }
    }
}

fn draw_players(state: &mut State) {
    for (idx, player) in state.players.iter().enumerate() {
        draw_rectangle(
            player.x as f32 * SPEED_MULTIPLIER,
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
