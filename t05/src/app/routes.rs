use crate::{app::commands::Commands, model::Player, repository::Repository};
use axum::{
    extract::{
        ws::{self, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{stream::StreamExt, SinkExt};
use itertools::Itertools;
use tracing::{error, info, warn};

pub(crate) fn initialize_router(state: Repository) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Repository>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Repository) {
    info!("Established connection with the client");
    let username = loop {
        if let Err(err) = socket
            .send(ws::Message::Text(
                "Please provide your username (non-empty)".to_owned(),
            ))
            .await
        {
            error!("Failed to send message to client: {err}");
            return;
        }

        let Some(received) = socket.recv().await else {
            info!("Client closed the connection");
            return;
        };

        let message = match received {
            Ok(message) => message,
            Err(err) => {
                error!("Failed to obtain message: {err}");
                return;
            }
        };

        let ws::Message::Text(username) = message else {
            continue;
        };
        let username = username.trim().to_owned();

        if username.is_empty() {
            continue;
        }

        break username;
    };

    let ulid = state.add_player(&username);
    info!("Client now has a name and ulid: '{username}':{ulid}");

    let client_task = tokio::spawn(async move {
        client_communication(socket, state, username, ulid).await;
    });

    if let Err(err) = client_task.await {
        error!("Failed to join client's task: {err}");
    }
}

#[allow(clippy::too_many_lines)]
async fn client_communication(
    socket: WebSocket,
    state: Repository,
    username: String,
    ulid: ulid::Ulid,
) {
    let (mut sender_ws, mut receiver_ws) = socket.split();
    let (sender_chat, mut receiver_chat) = state.get_chat();

    let (x, y) = state.get_player_position(ulid);
    let position = format!("({x}, {y}) - {username}");

    if let Err(err) = sender_ws.send(ws::Message::Text(position)).await {
        error!(
            username = username,
            ulid = ulid.to_string(),
            "Failed to send message to client: {err}"
        );
        return;
    }

    loop {
        tokio::select! {
            client_data = receiver_ws.next() => {
                match client_data {
                    Some(Ok(data)) => {
                        let ws::Message::Text(text) = data else {
                            warn!(username = username, ulid = ulid.to_string(), "Wrong message type received, continue");
                            continue;
                        };

                        // --- user actions using commands ---
                        let commands = match Commands::try_from(text.as_str()) {
                            Ok(command) => command,
                            Err(err) => {
                                error!(username = username, ulid = ulid.to_string(), "Incorrect command: '{err}'");
                                continue;
                            }
                        };

                        match commands {
                            Commands::Move(direction) => {
                                let (x, y) = state.move_player(ulid, &direction);
                                let position = format!("({x}, {y}) - {username}");
                                if let Err(err) = sender_ws.send(ws::Message::Text(position)).await {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                                    break;
                                }
                            },
                            Commands::Say(text) => {
                                let text = format!("{username}: {text}");
                                if let Err(err) = sender_chat.send(text) {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to chat: {err}");
                                    break;
                                };
                            },
                            Commands::WhoIsNearby => {
                                let nearby_players = state.get_nearby_players(ulid).into_iter();
                                let nearby_players = nearby_players.into_iter().format_with(
                                    ",\n",
                                    |(distance, Player { name, position, .. }), f| {
                                        f(&format_args!(
                                            "{:.2} units to ({}, {}) - {}",
                                            distance, position.0, position.1, name
                                        ))
                                    },
                                ).to_string();

                                if let Err(err) = sender_ws.send(ws::Message::Text(nearby_players)).await {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                                    break;
                                }
                            },
                        }
                        // --- user actions using commands ---

                        /*
                        // --- user actions using commands, previous version ---
                        let text = text.to_lowercase();
                        let text = text.split_whitespace().collect::<Vec<_>>();
                                                match text.as_slice() {
                            ["move", direction, ..] => {
                                let direction = match crate::dto::Direction::try_from(*direction) {
                                    Ok(direction) => {direction},
                                    Err(err) => {
                                        error!(username = username, ulid = ulid.to_string(), "Received incorrect data: {err}");
                                        continue;
                                    },
                                };
                                let (x, y) = state.move_player(ulid, &direction);
                                let position = format!("({x}, {y}) - {username}");
                                if let Err(err) = sender_ws.send(ws::Message::Text(position)).await {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                                    break;
                                }
                            },
                            ["say", text, ..] => {
                                let text = format!("{username}: {text}");
                                if let Err(err) = sender_chat.send(text) {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to chat: {err}");
                                    break;
                                };
                            },
                            ["whoisnearby", ..] => {
                                let nearby_players = state.get_nearby_players(ulid).into_iter();
                                let nearby_players = nearby_players.into_iter().format_with(
                                    ",\n",
                                    |(distance, Player { name, position, .. }), f| {
                                        f(&format_args!(
                                            "{:.2} units to ({}, {}) - {}",
                                            distance, position.0, position.1, name
                                        ))
                                    },
                                ).to_string();
                                if let Err(err) = sender_ws.send(ws::Message::Text(nearby_players)).await {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                                    break;
                                }
                            },
                            _ => {
                            },
                        }
                        // --- user actions using commands, previous version ---
                        */

                        /*
                        // --- user actions using jsons ---
                        let action = match serde_json::from_str::<Action>(&text) {
                            Ok(action) => action,
                            Err(err) => {
                                error!(username = username, ulid = ulid.to_string(), "Received incorrect data: {err}");
                                continue;
                            },
                        };

                        match action {
                            Action::Move { direction } => {
                                let (x, y) = state.move_player(ulid, &direction);
                                let position = format!("({x}, {y}) - {username}");
                                if let Err(err) = sender_ws.send(ws::Message::Text(position)).await {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                                    break;
                                }
                            }
                            Action::Say { text } => {
                                let text = format!("{username}: {text}");
                                if let Err(err) = sender_chat.send(text) {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to chat: {err}");
                                    break;
                                };
                            },
                            Action::WhoIsNearby {} => {
                                let nearby_players = state.get_nearby_players(ulid).into_iter();
                                let nearby_players = nearby_players.into_iter().format_with(
                                    ",\n",
                                    |(distance, Player { name, position, .. }), f| {
                                        f(&format_args!(
                                            "{:.2} units to ({}, {}) - {}",
                                            distance, position.0, position.1, name
                                        ))
                                    },
                                ).to_string();
                                if let Err(err) = sender_ws.send(ws::Message::Text(nearby_players)).await {
                                    error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                                    break;
                                }
                            },
                        }
                        // --- user actions using jsons ---
                        */

                    },
                    Some(Err(err)) => {
                        error!("Failed to get user input: {err}");
                        break;
                    },
                    None => {
                        info!("Client has ended connection");
                        break;
                    },
                }
            },
            chat_data = receiver_chat.recv() => {
                match chat_data {
                    Ok(data) => {
                        if let Err(err) = sender_ws.send(ws::Message::Text(data)).await {
                            error!(username = username, ulid = ulid.to_string(), "Failed to send message to client: {err}");
                            break;
                        }
                    },
                    Err(err) => {
                        error!(username = username, ulid = ulid.to_string(), "Failed to send message to chat: {err}");
                        break;
                    },
                }
            },
        }
    }

    info!(
        username = username,
        ulid = ulid.to_string(),
        "Client disconnected, removing player from state"
    );
    state.remove_player(ulid);
}
