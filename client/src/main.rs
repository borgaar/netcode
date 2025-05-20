use chrono::Utc;
use macroquad::{
    color::{BLACK, BLUE, GREEN},
    input::{get_keys_down, KeyCode},
    shapes::{draw_line, draw_rectangle},
    window::{clear_background, next_frame, screen_height, screen_width},
};
use netcode::{Player, State};

const GROUND_THICKNESS: f32 = 10.;
const PLAYER_SIZE: f32 = 30.;
const PLAYER_SPEED: f32 = 3.;
const GROUND_POSITION: f32 = 0.8;

#[macroquad::main("BasicShapes")]
async fn main() -> anyhow::Result<()> {
    let mut state = State {
        players: vec![Player {
            id: 0,
            x: 0.0,
            last_jump_at: Utc::now(),
        }],
    };

    loop {
        clear_background(BLACK);

        draw_line(
            0.,
            screen_height() * GROUND_POSITION,
            screen_width(),
            screen_height() * GROUND_POSITION,
            GROUND_THICKNESS,
            BLUE,
        );

        let keys_down = get_keys_down();
        if keys_down.iter().any(|key_code| key_code == &KeyCode::D) {
            state.players[0].x += PLAYER_SPEED;
        } else if keys_down.iter().any(|key_code| key_code == &KeyCode::A) {
            state.players[0].x -= PLAYER_SPEED;
        }

        for player in &state.players {
            draw_rectangle(
                player.x,
                (screen_height() * GROUND_POSITION) - (GROUND_THICKNESS / 2.) - PLAYER_SIZE,
                PLAYER_SIZE,
                PLAYER_SIZE,
                GREEN,
            );
        }

        next_frame().await
    }
}
