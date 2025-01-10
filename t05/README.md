## Task

Create a multiplayer online game using Axum and web sockets. Players can connect to the server, interact with each other in real time, and update the game state. The application must support the core logic of the game, player state management, and interaction via web sockets.

## Technical requirements

- Use the Axum framework to create a web server.
- Use web sockets for two-way communication between the server and clients.
- Implement the main functions of the game: connection, tracking the state of players, transmitting game events.
- Provide error handling and logging.

## Project structure

### Data models:

- Player (`id`, `name`, `position`)

- Game (`players`, `state`)

## API structure:

`GET /ws`: establish a web socket connection

## Game logic:

- Connecting and disconnecting players.
- Transmitting coordinates and actions of players.
- Updating the game state in real time.

## Launch
- Specify the `HOST` and `PORT` variables in the `.env` file.
- Connect via web sockets and specify your name.
- Available commands: `move { up | down | left | right }`, `say { any text }`, `whoisnearby`
