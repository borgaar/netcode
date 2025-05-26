# NetCube - Netcode client/server game

Made by Borgar Barland and Anders Noël Lothe Morille

## Introduction

This project aims to implement a simple 2D game to showcase/learn netcode functionality.
The game consists of a client side application written in Rust with the Macroquad minimal game engine. Players can join the game and move/jump around, seeing the other player's position in realtime. The server-side application - also written in Rust - exposes a SocketIO (WebSocket) API to handle realtime-connections with the client and global state updates.

## Functionality

- Graphical client-side game with movement and jumping
- Toggleable netcode functionality, namely interpolation, reconciliation and prediction
- Adjustable ping that is simulated on the client
- Sound effects for joining the game and adjusting ping
- Server-side application that provides a SocketIO API, and can handle multiple client connections simulatneously

## Roadmap/weaknesses

- Physics simulation; currently has constant movement speed in the X axis and virtual jumping based on formula/time
- Collisions; currently no collisions. Contact with ground based solely on min Y coordinates
-

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
cd server && cargo run --release
```

2. Run the client

```sh
cd client && cargo run --release
```

You can omit the releas flag to compile in debug mode.

## Running tests

Tests can be run with

```sh
cargo test
```