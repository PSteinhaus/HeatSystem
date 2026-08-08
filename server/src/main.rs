#[macro_use]
extern crate rocket;

mod game;
mod models;
mod state;
mod websocket;

use rocket::get;
use rocket::serde::json::Json;
use rocket::State;

use models::GameState;
use state::GameServer;

#[get("/")]
fn index() -> &'static str {
    "Heat System Server"
}

#[get("/game")]
async fn game_state(server: &State<GameServer>) -> Json<GameState> {
    Json(server.snapshot().await)
}

#[post("/game/start")]
async fn start_game(server: &State<GameServer>) -> &'static str {
    server.start_game().await;
    "ok"
}

#[launch]
fn rocket() -> _ {
    let game_server = GameServer::new();

    rocket::build()
        .manage(game_server)
        .mount("/", routes![
            index,
            game_state,
            start_game,
            websocket::websocket,
        ])
}