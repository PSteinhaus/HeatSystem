use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type HexId = String;
pub type PlayerId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hex {
    pub id: HexId,
    pub q: i32,
    pub r: i32,
    pub heat: i32,
    pub inscription: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// The player's stable identity within the game.
    ///
    /// For now this is the player's name. This deliberately allows
    /// a player to reconnect from another device by entering the
    /// same name again.
    pub id: PlayerId,

    pub name: String,
    pub hope: i32,
    pub color: String,
    pub position: Option<HexId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Mixed,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriticalType {
    CriticalSuccess,
    CriticalFailure,
    Upgrade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollResult {
    pub d1: u8,
    pub d2: u8,
    pub bonus: i32,
    pub heat: i32,
    pub modifier: i32,
    pub total: i32,
    pub outcome: Outcome,
    pub critical: Option<CriticalType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnPhase {
    Idle,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnState {
    pub phase: TurnPhase,
    pub active_player_id: Option<PlayerId>,
    pub bonus: i32,
    pub chain: Vec<HexId>,
    pub rolls: Vec<RollResult>,
    pub final_outcome: Option<Outcome>,
    pub heat_snapshot: HashMap<HexId, i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    Setup,
    Playing,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MusicTrackId {
    None,
    Ambient,
    Forest,
    Mystic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub status: GameStatus,
    pub hexes: HashMap<HexId, Hex>,
    pub hex_order: Vec<HexId>,
    pub players: HashMap<PlayerId, Player>,
    pub turn: TurnState,
    pub music_track: MusicTrackId,
}