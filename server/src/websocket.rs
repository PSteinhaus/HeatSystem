use rocket::futures::{SinkExt, StreamExt};
use rocket::get;
use rocket::State;
use rocket_ws as ws;
use serde::{Deserialize, Serialize};

use crate::models::{
    GameState,
    MusicTrackId,
    Player,
    RollResult,
};
use crate::state::{EndTurnResult, GameError, GameServer};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join {
        name: String,
    },

    StartGame,

    ResetGame,

    BeginTurn {
        player_id: String,
        bonus: i32,
    },

    RollHex {
        player_id: String,
        hex_id: String,
    },

    EndTurn {
        player_id: String,
    },

    SetHeat {
        hex_id: String,
        heat: i32,
    },

    SetHexCount {
        count: usize,
    },

    SwapHexHeat {
        hex_a: String,
        hex_b: String,
    },

    SetHope {
        player_id: String,
        hope: i32,
    },

    AdjustHope {
        player_id: String,
        delta: i32,
    },

    MovePlayer {
        player_id: String,
        hex_id: String,
    },

    SetInscription {
        hex_id: String,
        inscription: String,
    },

    SetMusicTrack {
        track: MusicTrackId,
    },

    RemovePlayer {
        player_id: String,
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

    RollResult {
        result: RollResult,
    },

    TurnEnded {
        hope_gained: i32,
        outcome: Option<crate::models::Outcome>,
    },

    Error {
        message: String,
    },
}

fn serialize_message(
    message: &ServerMessage,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(message)
}

async fn send_message(
    stream: &mut ws::stream::DuplexStream,
    message: &ServerMessage,
) -> Result<(), ws::result::Error> {
    let json = match serialize_message(message) {
        Ok(json) => json,

        Err(error) => {
            eprintln!(
                "Failed to serialize server message: {error}"
            );

            return Ok(());
        }
    };

    stream.send(ws::Message::text(json)).await
}

async fn send_error(
    stream: &mut ws::stream::DuplexStream,
    error: GameError,
) -> Result<(), ws::result::Error> {
    send_message(
        stream,
        &ServerMessage::Error {
            message: error.message().to_string(),
        },
    )
    .await
}

#[get("/ws")]
pub fn websocket(
    socket: ws::WebSocket,
    server: &State<GameServer>,
) -> ws::Channel<'static> {
    let server = server.inner().clone();

    socket.channel(move |mut stream| {
        Box::pin(async move {
            let mut updates = server.subscribe();

            /*
             * Send the current complete snapshot immediately.
             */
            let initial_state = server.snapshot().await;

            send_message(
                &mut stream,
                &ServerMessage::State {
                    state: initial_state,
                },
            )
            .await?;

            loop {
                tokio::select! {
                    /*
                     * Client -> server
                     */
                    incoming = stream.next() => {
                        let Some(incoming) = incoming else {
                            break;
                        };

                        let message = match incoming {
                            Ok(message) => message,

                            Err(error) => {
                                eprintln!(
                                    "WebSocket receive error: {error}"
                                );
                                break;
                            }
                        };

                        let text = match message.into_text() {
                            Ok(text) => text,

                            Err(error) => {
                                send_message(
                                    &mut stream,
                                    &ServerMessage::Error {
                                        message: format!(
                                            "Invalid WebSocket message: {error}"
                                        ),
                                    },
                                )
                                .await?;

                                continue;
                            }
                        };

                        let command =
                            match serde_json::from_str::<ClientMessage>(&text) {
                                Ok(command) => command,

                                Err(error) => {
                                    send_message(
                                        &mut stream,
                                        &ServerMessage::Error {
                                            message: format!(
                                                "Invalid command: {error}"
                                            ),
                                        },
                                    )
                                    .await?;

                                    eprintln!(
                                        "Invalid client command: {error}"
                                    );

                                    continue;
                                }
                            };

                        match command {
                            ClientMessage::Join { name } => {
                                match server.join(name).await {
                                    Ok(player) => {
                                        send_message(
                                            &mut stream,
                                            &ServerMessage::Joined {
                                                player,
                                            },
                                        )
                                        .await?;
                                    }

                                    Err(error) => {
                                        send_error(
                                            &mut stream,
                                            error,
                                        )
                                        .await?;
                                    }
                                }
                            }

                            ClientMessage::StartGame => {
                                server.start_game().await;
                            }

                            ClientMessage::ResetGame => {
                                server.reset_game().await;
                            }

                            ClientMessage::SetHexCount { count } => {
                                if let Err(error) =
                                    server.set_hex_count(count).await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::BeginTurn {
                                player_id,
                                bonus,
                            } => {
                                match server
                                    .begin_turn(
                                        &player_id,
                                        bonus,
                                    )
                                    .await
                                {
                                    Ok(()) => {}

                                    Err(error) => {
                                        send_error(
                                            &mut stream,
                                            error,
                                        )
                                        .await?;
                                    }
                                }
                            }

                            ClientMessage::RollHex {
                                player_id,
                                hex_id,
                            } => {
                                match server
                                    .roll_for_hex(
                                        &player_id,
                                        &hex_id,
                                    )
                                    .await
                                {
                                    Ok(result) => {
                                        /*
                                         * The complete state will also
                                         * be broadcast through `updates`.
                                         *
                                         * This RollResult message is useful
                                         * for the requesting client if we
                                         * later want to distinguish the
                                         * immediate action result from the
                                         * state snapshot.
                                         */
                                        send_message(
                                            &mut stream,
                                            &ServerMessage::RollResult {
                                                result,
                                            },
                                        )
                                        .await?;
                                    }

                                    Err(error) => {
                                        send_error(
                                            &mut stream,
                                            error,
                                        )
                                        .await?;
                                    }
                                }
                            }

                            ClientMessage::EndTurn {
                                player_id,
                            } => {
                                match server
                                    .end_turn(
                                        &player_id,
                                    )
                                    .await
                                {
                                    Ok(result) => {
                                        send_message(
                                            &mut stream,
                                            &ServerMessage::TurnEnded {
                                                hope_gained:
                                                    result.hope_gained,
                                                outcome:
                                                    result.outcome,
                                            },
                                        )
                                        .await?;
                                    }

                                    Err(error) => {
                                        send_error(
                                            &mut stream,
                                            error,
                                        )
                                        .await?;
                                    }
                                }
                            }

                            ClientMessage::SetHeat {
                                hex_id,
                                heat,
                            } => {
                                if let Err(error) =
                                    server
                                        .set_heat(
                                            &hex_id,
                                            heat,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::SwapHexHeat {
                                hex_a,
                                hex_b,
                            } => {
                                if let Err(error) =
                                    server
                                        .swap_hex_heat(
                                            &hex_a,
                                            &hex_b,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::SetHope {
                                player_id,
                                hope,
                            } => {
                                if let Err(error) =
                                    server
                                        .set_hope(
                                            &player_id,
                                            hope,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::AdjustHope {
                                player_id,
                                delta,
                            } => {
                                if let Err(error) =
                                    server
                                        .adjust_hope(
                                            &player_id,
                                            delta,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::MovePlayer {
                                player_id,
                                hex_id,
                            } => {
                                if let Err(error) =
                                    server
                                        .move_player(
                                            &player_id,
                                            &hex_id,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::SetInscription {
                                hex_id,
                                inscription,
                            } => {
                                if let Err(error) =
                                    server
                                        .set_inscription(
                                            &hex_id,
                                            inscription,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }

                            ClientMessage::SetMusicTrack {
                                track,
                            } => {
                                server
                                    .set_music_track(track)
                                    .await;
                            }

                            ClientMessage::RemovePlayer {
                                player_id,
                            } => {
                                if let Err(error) =
                                    server
                                        .remove_player(
                                            &player_id,
                                        )
                                        .await
                                {
                                    send_error(
                                        &mut stream,
                                        error,
                                    )
                                    .await?;
                                }
                            }
                        }
                    }

                    /*
                     * Server -> client
                     */
                    update = updates.recv() => {
                        match update {
                            Ok(state) => {
                                send_message(
                                    &mut stream,
                                    &ServerMessage::State {
                                        state,
                                    },
                                )
                                .await?;
                            }

                            Err(
                                tokio::sync::broadcast::error::RecvError::Lagged(_)
                            ) => {
                                /*
                                 * Because our protocol uses complete
                                 * snapshots, recovery is trivial.
                                 */
                                let state =
                                    server.snapshot().await;

                                send_message(
                                    &mut stream,
                                    &ServerMessage::State {
                                        state,
                                    },
                                )
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