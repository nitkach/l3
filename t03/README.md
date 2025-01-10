## Task

Build a multi-user online chat where users can join rooms, send messages, and receive messages in real time. The chat should support multiple rooms, and users can only send messages to rooms they have joined. The implementation should use `axum` for the web server, and various sync structures and async features of Rust.

## Technical requirements

- Use the Axum framework to build the web server
- Use `Arc`, `Mutex`, `RwLock`, `Box`, channels, and `tokio` for state management and async processing
- Use `DashMap` to store user and room data
- Use atomic variables (`AtomicUsize`) to track the number of users

## Project structure

#### Data models:

- User
- Room
- Message

## API structure:

- `POST /join`: join a room.
- `POST /leave`: leave a room.
- `POST /send`: send a message to a room.
- `GET /messages`: get messages from the room.
