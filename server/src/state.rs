use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::game::{self, PLAYER_COLORS, STARTING_HOPE};
use crate::models::{GameState, GameStatus, Player, PlayerId};

const BROADCAST_CAPACITY: usize = 64;

#[derive(Debug, Clone)]
pub struct GameServer {
    state: Arc<RwLock<GameState>>,
    updates: broadcast::Sender<GameState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    GameNotPlaying,
    InvalidPlayerName,
    PlayerNameAlreadyTaken,
}

impl GameError {
    pub fn message(&self) -> &'static str {
        match self {
            GameError::GameNotPlaying => {
                "The game is not currently accepting players."
            }
            GameError::InvalidPlayerName => {
                "Player name must not be empty."
            }
            GameError::PlayerNameAlreadyTaken => {
                "That player name is already taken."
            }
        }
    }
}

impl GameServer {
    pub fn new() -> Self {
        let (updates, _) = broadcast::channel(BROADCAST_CAPACITY);

        Self {
            state: Arc::new(RwLock::new(game::default_game_state())),
            updates,
        }
    }

    pub async fn snapshot(&self) -> GameState {
        self.state.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GameState> {
        self.updates.subscribe()
    }

    async fn publish(&self) {
        let state = self.snapshot().await;

        /*
         * It is okay if there are currently no subscribers.
         * broadcast::Sender::send() returns an error in that case,
         * but the state itself has already been changed successfully.
         */
        let _ = self.updates.send(state);
    }

    pub async fn join(
        &self,
        name: String,
    ) -> Result<Player, GameError> {
        let name = name.trim().to_string();

        if name.is_empty() {
            return Err(GameError::InvalidPlayerName);
        }

        let mut state = self.state.write().await;

        if !matches!(state.status, GameStatus::Playing) {
            return Err(GameError::GameNotPlaying);
        }

        /*
         * Names are unique for now.
         *
         * Later, once we introduce actual accounts/sessions,
         * this won't be our identity mechanism anymore.
         */
        if state
            .players
            .values()
            .any(|player| player.name == name)
        {
            return Err(GameError::PlayerNameAlreadyTaken);
        }

        let player_id = Uuid::new_v4().to_string();

        let color_index =
            state.players.len() % PLAYER_COLORS.len();

        let occupied_positions = state
            .players
            .values()
            .filter_map(|player| player.position.as_ref())
            .collect::<std::collections::HashSet<_>>();

        let free_hex = state
            .hex_order
            .iter()
            .find(|hex_id| !occupied_positions.contains(hex_id))
            .cloned();

        let player = Player {
            id: player_id.clone(),
            name,
            hope: STARTING_HOPE,
            color: PLAYER_COLORS[color_index].to_string(),
            position: free_hex,
        };

        state.players.insert(player_id, player.clone());

        drop(state);

        self.publish().await;

        Ok(player)
    }

    /*
     * Used by tests and future game-management routes.
     */
    pub async fn start_game(&self) {
        {
            let mut state = self.state.write().await;

            state.status = GameStatus::Playing;
            state.players.clear();
            state.turn = crate::models::TurnState {
                phase: crate::models::TurnPhase::Idle,
                active_player_id: None,
                bonus: 0,
                chain: Vec::new(),
                rolls: Vec::new(),
                final_outcome: None,
                heat_snapshot: std::collections::HashMap::new(),
            };
            state.music_track = crate::models::MusicTrackId::None;
        }

        self.publish().await;
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn joining_game_creates_player() {
        let server = GameServer::new();

        server.start_game().await;

        let player = server
            .join("Alice".to_string())
            .await
            .expect("player should join");

        assert_eq!(player.name, "Alice");
        assert_eq!(player.hope, STARTING_HOPE);
        assert!(player.position.is_some());

        let state = server.snapshot().await;

        assert_eq!(state.players.len(), 1);
        assert!(state.players.contains_key(&player.id));
    }

    #[tokio::test]
    async fn duplicate_names_are_rejected() {
        let server = GameServer::new();

        server.start_game().await;

        server
            .join("Alice".to_string())
            .await
            .expect("first player should join");

        let result = server.join("Alice".to_string()).await;

        assert_eq!(
            result,
            Err(GameError::PlayerNameAlreadyTaken)
        );
    }

    #[tokio::test]
    async fn joining_before_game_start_is_rejected() {
        let server = GameServer::new();

        let result = server.join("Alice".to_string()).await;

        assert_eq!(
            result,
            Err(GameError::GameNotPlaying)
        );
    }
}