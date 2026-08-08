use rocket::futures::{SinkExt, StreamExt};
use rocket::get;
use rocket::State;
use rocket_ws as ws;

use serde::{Deserialize, Serialize};

use crate::models::{GameState, Player};
use crate::state::GameServer;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join {
        name: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    State {
        state: GameState,
    },

    Joined {
        player: Player,
    },

    Error {
        message: String,
    },
}

fn serialize_message(message: &ServerMessage) -> Result<String, serde_json::Error> {
    serde_json::to_string(message)
}

#[get("/ws")]
pub fn websocket(
    socket: ws::WebSocket,
    server: &State<GameServer>,
) -> ws::Channel<'static> {
    let server = server.inner().clone();

    socket.channel(move |mut stream| {
        Box::pin(async move {
            /*
             * Subscribe BEFORE taking our initial snapshot.
             *
             * This means that state changes occurring immediately
             * after connection establishment are not silently missed.
             */
            let mut updates = server.subscribe();

            /*
             * Every connection receives the current state immediately.
             */
            let initial_state = server.snapshot().await;

            let initial_message = ServerMessage::State {
                state: initial_state,
            };

            match serialize_message(&initial_message) {
                Ok(json) => {
                    stream.send(ws::Message::text(json)).await?;
                }
                Err(error) => {
                    eprintln!("Failed to serialize initial state: {error}");
                    return Ok(());
                }
            }

            /*
             * Process both:
             *
             *   client -> server messages
             *
             * and:
             *
             *   server -> all clients state broadcasts
             *
             * simultaneously.
             */
            loop {
                tokio::select! {
                    incoming = stream.next() => {
                        let Some(incoming) = incoming else {
                            break;
                        };

                        let message = match incoming {
                            Ok(message) => message,
                            Err(error) => {
                                eprintln!("WebSocket receive error: {error}");
                                break;
                            }
                        };

                        let text = match message.into_text() {
                            Ok(text) => text,
                            Err(error) => {
                                let response = ServerMessage::Error {
                                    message: format!(
                                        "Invalid WebSocket message: {error}"
                                    ),
                                };

                                if let Ok(json) = serialize_message(&response) {
                                    let _ = stream
                                        .send(ws::Message::text(json))
                                        .await;
                                }

                                continue;
                            }
                        };

                        let command =
                            match serde_json::from_str::<ClientMessage>(&text) {
                                Ok(command) => command,
                                Err(error) => {
                                    let response = ServerMessage::Error {
                                        message: format!(
                                            "Invalid command: {error}"
                                        ),
                                    };

                                    if let Ok(json) =
                                        serialize_message(&response)
                                    {
                                        let _ = stream
                                            .send(ws::Message::text(json))
                                            .await;
                                    }

                                    continue;
                                }
                            };

                        match command {
                            ClientMessage::Join { name } => {
                                match server.join(name).await {
                                    Ok(player) => {
                                        let response =
                                            ServerMessage::Joined {
                                                player,
                                            };

                                        if let Ok(json) =
                                            serialize_message(&response)
                                        {
                                            stream
                                                .send(ws::Message::text(json))
                                                .await?;
                                        }
                                    }

                                    Err(error) => {
                                        let response =
                                            ServerMessage::Error {
                                                message: error
                                                    .message()
                                                    .to_string(),
                                            };

                                        if let Ok(json) =
                                            serialize_message(&response)
                                        {
                                            stream
                                                .send(ws::Message::text(json))
                                                .await?;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    update = updates.recv() => {
                        match update {
                            Ok(state) => {
                                let response =
                                    ServerMessage::State { state };

                                let json =
                                    match serialize_message(&response) {
                                        Ok(json) => json,
                                        Err(error) => {
                                            eprintln!(
                                                "Failed to serialize state: {error}"
                                            );
                                            continue;
                                        }
                                    };

                                stream
                                    .send(ws::Message::text(json))
                                    .await?;
                            }

                            Err(
                                tokio::sync::broadcast::error::RecvError::Lagged(_)
                            ) => {
                                /*
                                 * We missed one or more state updates.
                                 *
                                 * Since GameState is authoritative and
                                 * completely serializable, simply send the
                                 * current complete snapshot.
                                 */
                                let state = server.snapshot().await;

                                let response =
                                    ServerMessage::State { state };

                                let json =
                                    match serialize_message(&response) {
                                        Ok(json) => json,
                                        Err(error) => {
                                            eprintln!(
                                                "Failed to serialize recovery state: {error}"
                                            );
                                            continue;
                                        }
                                    };

                                stream
                                    .send(ws::Message::text(json))
                                    .await?;
                            }

                            Err(
                                tokio::sync::broadcast::error::RecvError::Closed
                            ) => {
                                break;
                            }
                        }
                    }
                }
            }

            Ok(())
        })
    })
}