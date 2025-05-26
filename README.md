# NetCube - Netcode client/server game

Made by Borgar Barland and Anders Noël Lothe Morille

## Introduction

This project aims to implement a simple 2D game to showcase/learn netcode functionality.
The game consists of a client side application written in Rust with the Macroquad minimal game engine. Players can join the game and move/jump around, seeing the other player's position in realtime. The server-side application - also written in Rust - exposes a SocketIO (WebSocket) API to handle realtime-connections with the client and global state updates.

## Functionality

- Graphical client-side game with movement and jumping
- Toggleable netcode functionality, namely interpolation, reconciliation and prediction
- Adjustable ping that is simulated on the client for both sending and receiving packets.
- Sound effects for joining the game and adjusting ping
- Server-side application that provides a SocketIO API, and can handle multiple client connections simultaneously

## Roadmap/weaknesses

- Physics simulation; currently has constant movement speed in the X axis and virtual and jumping where height is calculated based on the time of the jump.
- Collisions; currently no collisions. Contact with ground based solely on min Y coordinates

## Dependencies

Packages used in the project.
The project is divided into 3 crates (rust packages): client, server and netcode.
Dependencies that are shared between the three reside in the workspace dependencies.

### Workspace

- serde: Generic serialization/deserialization library
- serde_json: Serialization of JSON values
- anyhow: Thin-pointer error types for coercible error fallback
- chrono: Date/time/duration handling with Timezone
- thiserror: Library error handling using enums

### Server

- axum: Web-API server library. Bindings for TCP/HTTP
- socketioxide: SocketIO library. Bindings for SocketIO/WebSocket
- tokio: Asynchronous runtime for async/await

### Client

macroquad: Minimal game engine for audio, graphics and game loop
rust_socketio: SocketIO client

## Installation

This project requires [Rust](https://www.rust-hlang.org/). The project was written in with rustc v1.87.0.
Rust can be installed using [Rustup](https://www.rust-lang.org/tools/install).
The latest stable release should suffice.

## Running the project

1. Run the server

```sh
cargo run --release --bin server
```

2. Run the client (must run the server first)

```sh
cargo run --release --bin client
```

You can omit the release flag to compile in debug mode, but this might affect performance.

## How to use

### Keybinds

- `Space` - Join the game
- `W` - Jump
- `A` - Move left
- `D` - Move right
- `J` - Reduce ping
- `K` - Increase ping
- `I` - Toggle interpolation
- `R` - Toggle reconciliation
- `P` - Toggle prediction

### Adjust server update rate

The server's update rate can be changed by increasing or decreasing the `STATE_UPDATE_INTERVAL` constant at the top of the `server/src/main.rs` file, and restarting the server.
This constant represents how many milliseconds the server waits before sending a new state update to the clients. It is recommended to keep this value above 15ms.

## Running tests

Tests can be run with

```sh
cargo test
```

