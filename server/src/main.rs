#[macro_use]
extern crate rocket;

mod game;
mod models;
mod state;
mod websocket;

use rocket::get;
use rocket::post;
use rocket::serde::json::Json;
use rocket::State;

use models::GameState;
use state::GameServer;

#[get("/")]
fn index() -> &'static str {
    "Heat System Server"
}

#[get("/game")]
async fn game_state(
    server: &State<GameServer>,
) -> Json<GameState> {
    Json(server.snapshot().await)
}

/*
 * Temporary development endpoint.
 *
 * Once we have the concept of a game host/session, this will be
 * replaced by the proper game-management mechanism.
 */
#[post("/game/start")]
async fn start_game(
    server: &State<GameServer>,
) -> &'static str {
    server.start_game().await;

    "ok"
}

/*
 * Temporary development endpoint.
 */
#[post("/game/reset")]
async fn reset_game(
    server: &State<GameServer>,
) -> &'static str {
    server.reset_game().await;

    "ok"
}

#[launch]
fn rocket() -> _ {
    let game_server = GameServer::new();

    rocket::build()
        .manage(game_server)
        .mount(
            "/api",
            routes![
                index,
                game_state,
                start_game,
                reset_game,
                websocket::websocket,
            ],
        )
}