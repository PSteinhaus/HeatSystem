use std::collections::HashMap;
use std::sync::Arc;

use rand::random_range;
use tokio::sync::{broadcast, RwLock};

use crate::game::{
    are_neighbors,
    bonus_cost,
    calculate_hope_gained,
    calculate_outcome,
    default_game_state,
    occupied_positions,
    PLAYER_COLORS,
    MAX_BONUS,
    MAX_HEAT,
    STARTING_HOPE,
};
use crate::models::{
    GameState,
    GameStatus,
    HexId,
    MusicTrackId,
    Outcome,
    Player,
    PlayerId,
    RollResult,
    TurnPhase,
    TurnState,
};

const BROADCAST_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    GameNotPlaying,
    InvalidPlayerName,
    InvalidBonus,
    PlayerNotFound,
    NotYourTurn,
    NoActiveTurn,
    PlayerHasNoPosition,
    HexNotFound,
    HexNotAdjacent,
    HexAlreadyVisited,
    NotEnoughHope,
    TurnAlreadyActive,
    TurnHasFailed,
    NoRolls,
}

impl GameError {
    pub fn message(&self) -> &'static str {
        match self {
            GameError::GameNotPlaying => {
                "The game is not currently active."
            }

            GameError::InvalidPlayerName => {
                "Player name must not be empty."
            }

            GameError::InvalidBonus => {
                "Bonus must be between 0 and 8."
            }

            GameError::PlayerNotFound => {
                "Player was not found."
            }

            GameError::NotYourTurn => {
                "It is not your turn."
            }

            GameError::NoActiveTurn => {
                "There is currently no active turn."
            }

            GameError::PlayerHasNoPosition => {
                "The player has no position."
            }

            GameError::HexNotFound => {
                "The requested hex does not exist."
            }

            GameError::HexNotAdjacent => {
                "The requested hex is not adjacent to the current position."
            }

            GameError::HexAlreadyVisited => {
                "That hex has already been visited during this turn."
            }

            GameError::NotEnoughHope => {
                "The player does not have enough hope."
            }

            GameError::TurnAlreadyActive => {
                "A turn is already active."
            }

            GameError::TurnHasFailed => {
                "The turn has already failed."
            }

            GameError::NoRolls => {
                "The current turn contains no rolls."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EndTurnResult {
    pub hope_gained: i32,
    pub outcome: Option<Outcome>,
}

#[derive(Debug, Clone)]
pub struct GameServer {
    state: Arc<RwLock<GameState>>,
    updates: broadcast::Sender<GameState>,
}

impl GameServer {
    pub fn new() -> Self {
        let (updates, _) = broadcast::channel(BROADCAST_CAPACITY);

        Self {
            state: Arc::new(RwLock::new(default_game_state())),
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

        let _ = self.updates.send(state);
    }

    // ---------------------------------------------------------------------
    // GAME LIFECYCLE
    // ---------------------------------------------------------------------

    pub async fn start_game(&self) {
        {
            let mut state = self.state.write().await;

            state.status = GameStatus::Playing;

            state.players.clear();

            state.turn = idle_turn();

            state.music_track = MusicTrackId::None;
        }

        self.publish().await;
    }

    pub async fn reset_game(&self) {
        {
            let mut state = self.state.write().await;

            *state = default_game_state();
        }

        self.publish().await;
    }

    // ---------------------------------------------------------------------
    // PLAYERS
    // ---------------------------------------------------------------------

    /// Join the game using the player's name as their stable ID.
    ///
    /// If the name already exists, this is treated as a reconnect and
    /// returns the existing player rather than creating a duplicate.
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
         * The player name is deliberately the player ID.
         *
         * This means:
         *
         *   Alice → player id "Alice"
         *
         * Reconnecting from another device with "Alice" retrieves
         * the same Player object.
         */
        if let Some(existing) = state.players.get(&name) {
            return Ok(existing.clone());
        }

        let color_idx =
            state.players.len() % PLAYER_COLORS.len();

        let occupied = occupied_positions(
            state.players.values(),
        );

        let free_hex = state
            .hex_order
            .iter()
            .find(|id| !occupied.contains(*id))
            .cloned();

        let player = Player {
            id: name.clone(),
            name: name.clone(),
            hope: STARTING_HOPE,
            color: PLAYER_COLORS[color_idx].to_string(),
            position: free_hex,
        };

        state.players.insert(name, player.clone());

        drop(state);

        self.publish().await;

        Ok(player)
    }

    pub async fn remove_player(
        &self,
        player_id: &str,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if state.players.remove(player_id).is_none() {
            return Err(GameError::PlayerNotFound);
        }

        drop(state);

        self.publish().await;

        Ok(())
    }

    // ---------------------------------------------------------------------
    // TURN MANAGEMENT
    // ---------------------------------------------------------------------

    pub async fn begin_turn(
        &self,
        player_id: &str,
        bonus: i32,
    ) -> Result<(), GameError> {
        if !(0..=MAX_BONUS).contains(&bonus) {
            return Err(GameError::InvalidBonus);
        }

        let mut state = self.state.write().await;

        if !matches!(state.status, GameStatus::Playing) {
            return Err(GameError::GameNotPlaying);
        }

        if matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::TurnAlreadyActive);
        }

        let player = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if player.position.is_none() {
            return Err(GameError::PlayerHasNoPosition);
        }

        let cost = bonus_cost(bonus);

        if player.hope < cost {
            return Err(GameError::NotEnoughHope);
        }

        /*
         * Snapshot all heat values BEFORE changing anything else.
         */
        let heat_snapshot = state
            .hexes
            .values()
            .map(|hex| (hex.id.clone(), hex.heat))
            .collect::<HashMap<_, _>>();

        /*
         * We need to borrow the player mutably only after obtaining
         * the snapshot.
         */
        let player = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        player.hope -= cost;

        state.turn = TurnState {
            phase: TurnPhase::Active,

            active_player_id: Some(player_id.to_string()),

            bonus,

            chain: Vec::new(),

            rolls: Vec::new(),

            final_outcome: None,

            heat_snapshot,
        };

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn roll_for_hex(
        &self,
        player_id: &str,
        hex_id: &str,
    ) -> Result<RollResult, GameError> {
        let mut state = self.state.write().await;

        if !matches!(state.status, GameStatus::Playing) {
            return Err(GameError::GameNotPlaying);
        }

        if !matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::NoActiveTurn);
        }

        if state.turn.active_player_id.as_deref()
            != Some(player_id)
        {
            return Err(GameError::NotYourTurn);
        }

        /*
         * A failure ends the rolling portion of the turn.
         *
         * The player still has to explicitly end the turn,
         * matching the current Vue UI.
         */
        if state.turn.final_outcome == Some(Outcome::Failure) {
            return Err(GameError::TurnHasFailed);
        }

        let player = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let player_position = player
            .position
            .clone()
            .ok_or(GameError::PlayerHasNoPosition)?;

        let target_hex = state
            .hexes
            .get(hex_id)
            .ok_or(GameError::HexNotFound)?;

        let current_hex_id = state
            .turn
            .chain
            .last()
            .cloned()
            .unwrap_or(player_position);

        let current_hex = state
            .hexes
            .get(&current_hex_id)
            .ok_or(GameError::HexNotFound)?;

        if !are_neighbors(current_hex, target_hex) {
            return Err(GameError::HexNotAdjacent);
        }

        if state.turn.chain.iter().any(|id| id == hex_id) {
            return Err(GameError::HexAlreadyVisited);
        }

        let is_continuation = !state.turn.rolls.is_empty();

        let cost = bonus_cost(state.turn.bonus);

        /*
         * Every roll after the first costs the bonus again.
         */
        if is_continuation && player.hope < cost {
            return Err(GameError::NotEnoughHope);
        }

        let bonus = state.turn.bonus;

        let heat = target_hex.heat;

        /*
         * Dice are generated exclusively by the authoritative server.
         *
         * rand 0.10 uses random_range rather than the older gen_range API.
         */
        let d1: u8 = random_range(1..=6);
        let d2: u8 = random_range(1..=6);

        let result =
            calculate_outcome(d1, d2, bonus, heat);

        /*
         * Apply player changes.
         */
        let player = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if is_continuation {
            player.hope -= cost;
        }

        player.position = Some(hex_id.to_string());

        /*
         * Apply turn changes.
         */
        state.turn.chain.push(hex_id.to_string());

        state.turn.rolls.push(result.clone());

        if result.outcome == Outcome::Failure {
            state.turn.final_outcome = Some(Outcome::Failure);
        }

        drop(state);

        self.publish().await;

        Ok(result)
    }

    pub async fn end_turn(
        &self,
        player_id: &str,
    ) -> Result<EndTurnResult, GameError> {
        let mut state = self.state.write().await;

        if !matches!(state.status, GameStatus::Playing) {
            return Err(GameError::GameNotPlaying);
        }

        if !matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::NoActiveTurn);
        }

        if state.turn.active_player_id.as_deref()
            != Some(player_id)
        {
            return Err(GameError::NotYourTurn);
        }

        let last_outcome = state
            .turn
            .rolls
            .last()
            .map(|roll| roll.outcome);

        let chain_heat_sum = state
            .turn
            .chain
            .iter()
            .map(|hex_id| {
                state
                    .turn
                    .heat_snapshot
                    .get(hex_id)
                    .copied()
                    .unwrap_or(0)
                    .max(0)
            })
            .sum::<i32>();

        let hope_gained = match last_outcome {
            Some(outcome) => {
                calculate_hope_gained(
                    outcome,
                    chain_heat_sum,
                )
            }

            None => 0,
        };

        let player = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        player.hope += hope_gained;

        let result = EndTurnResult {
            hope_gained,
            outcome: last_outcome,
        };

        state.turn = idle_turn();

        drop(state);

        self.publish().await;

        Ok(result)
    }

    // ---------------------------------------------------------------------
    // ADMINISTRATIVE / SETUP OPERATIONS
    // ---------------------------------------------------------------------

    pub async fn set_hex_count(
        &self,
        count: usize,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if !matches!(state.status, GameStatus::Setup) {
            return Err(GameError::GameNotPlaying);
        }

        let hexes_vec = crate::game::generate_hexes(count);

        state.hexes.clear();
        state.hex_order.clear();

        for hex in hexes_vec {
            state.hex_order.push(hex.id.clone());
            state.hexes.insert(hex.id.clone(), hex);
        }

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn set_heat(
        &self,
        hex_id: &str,
        heat: i32,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::TurnAlreadyActive);
        }

        let hex = state
            .hexes
            .get_mut(hex_id)
            .ok_or(GameError::HexNotFound)?;

        hex.heat = heat.clamp(-MAX_HEAT, MAX_HEAT);

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn swap_hex_heat(
        &self,
        hex_a: &str,
        hex_b: &str,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::TurnAlreadyActive);
        }

        /*
         * Handle the possibility that both IDs are identical cleanly.
         */
        if hex_a == hex_b {
            if !state.hexes.contains_key(hex_a) {
                return Err(GameError::HexNotFound);
            }

            return Ok(());
        }

        let heat_a = state
            .hexes
            .get(hex_a)
            .ok_or(GameError::HexNotFound)?
            .heat;

        let heat_b = state
            .hexes
            .get(hex_b)
            .ok_or(GameError::HexNotFound)?
            .heat;

        state
            .hexes
            .get_mut(hex_a)
            .ok_or(GameError::HexNotFound)?
            .heat = heat_b;

        state
            .hexes
            .get_mut(hex_b)
            .ok_or(GameError::HexNotFound)?
            .heat = heat_a;

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn set_hope(
        &self,
        player_id: &str,
        hope: i32,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::TurnAlreadyActive);
        }

        let player = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        player.hope = hope.max(0);

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn adjust_hope(
        &self,
        player_id: &str,
        delta: i32,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if matches!(state.turn.phase, TurnPhase::Active) {
            return Err(GameError::TurnAlreadyActive);
        }

        let player = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        player.hope = (player.hope + delta).max(0);

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn move_player(
        &self,
        player_id: &str,
        hex_id: &str,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        if matches!(state.turn.phase, TurnPhase::Active)
            && state.turn.active_player_id.as_deref()
                != Some(player_id)
        {
            return Err(GameError::NotYourTurn);
        }

        if !state.hexes.contains_key(hex_id) {
            return Err(GameError::HexNotFound);
        }

        let player = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        player.position = Some(hex_id.to_string());

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn set_inscription(
        &self,
        hex_id: &str,
        inscription: String,
    ) -> Result<(), GameError> {
        let mut state = self.state.write().await;

        let hex = state
            .hexes
            .get_mut(hex_id)
            .ok_or(GameError::HexNotFound)?;

        hex.inscription = inscription.chars().take(100).collect();

        drop(state);

        self.publish().await;

        Ok(())
    }

    pub async fn set_music_track(
        &self,
        track: MusicTrackId,
    ) {
        let mut state = self.state.write().await;

        state.music_track = track;

        drop(state);

        self.publish().await;
    }
}

fn idle_turn() -> TurnState {
    TurnState {
        phase: TurnPhase::Idle,
        active_player_id: None,
        bonus: 0,
        chain: Vec::new(),
        rolls: Vec::new(),
        final_outcome: None,
        heat_snapshot: HashMap::new(),
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
    async fn game_starts_in_setup() {
        let server = GameServer::new();

        let state = server.snapshot().await;

        assert_eq!(state.status, GameStatus::Setup);
        assert!(state.players.is_empty());
        assert_eq!(state.hexes.len(), 14);
    }

    #[tokio::test]
    async fn starting_game_changes_status() {
        let server = GameServer::new();

        server.start_game().await;

        let state = server.snapshot().await;

        assert_eq!(state.status, GameStatus::Playing);
    }

    #[tokio::test]
    async fn joining_creates_player_with_name_as_id() {
        let server = GameServer::new();

        server.start_game().await;

        let player = server
            .join("Alice".into())
            .await
            .unwrap();

        assert_eq!(player.id, "Alice");
        assert_eq!(player.name, "Alice");
        assert_eq!(player.hope, STARTING_HOPE);
        assert!(player.position.is_some());

        let state = server.snapshot().await;

        assert!(state.players.contains_key("Alice"));
    }

    #[tokio::test]
    async fn reconnecting_returns_existing_player() {
        let server = GameServer::new();

        server.start_game().await;

        let first = server
            .join("Alice".into())
            .await
            .unwrap();

        /*
         * Change some persistent player state to prove that the
         * second join returns the existing object.
         */
        server
            .set_hope("Alice", 42)
            .await
            .unwrap();

        let second = server
            .join("Alice".into())
            .await
            .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.hope, 42);

        let state = server.snapshot().await;

        assert_eq!(state.players.len(), 1);
    }

    #[tokio::test]
    async fn beginning_turn_deducts_bonus_cost() {
        let server = GameServer::new();

        server.start_game().await;

        server
            .join("Alice".into())
            .await
            .unwrap();

        server
            .begin_turn("Alice", 2)
            .await
            .unwrap();

        let state = server.snapshot().await;

        assert_eq!(
            state.players["Alice"].hope,
            STARTING_HOPE - 4
        );

        assert_eq!(
            state.turn.active_player_id.as_deref(),
            Some("Alice")
        );

        assert_eq!(state.turn.bonus, 2);
        assert_eq!(state.turn.chain.len(), 0);
        assert_eq!(state.turn.rolls.len(), 0);
        assert_eq!(
            state.turn.heat_snapshot.len(),
            state.hexes.len()
        );
    }

    #[tokio::test]
    async fn cannot_begin_two_turns() {
        let server = GameServer::new();

        server.start_game().await;

        server
            .join("Alice".into())
            .await
            .unwrap();

        server
            .begin_turn("Alice", 1)
            .await
            .unwrap();

        let result =
            server.begin_turn("Alice", 1).await;

        assert_eq!(
            result,
            Err(GameError::TurnAlreadyActive)
        );
    }

    #[tokio::test]
    async fn cannot_begin_turn_without_enough_hope() {
        let server = GameServer::new();

        server.start_game().await;

        server
            .join("Alice".into())
            .await
            .unwrap();

        /*
         * Bonus 4 costs 16, but starting hope is 10.
         */
        let result =
            server.begin_turn("Alice", 4).await;

        assert_eq!(
            result,
            Err(GameError::NotEnoughHope)
        );
    }

    #[tokio::test]
    async fn end_turn_restores_idle_state() {
        let server = GameServer::new();

        server.start_game().await;

        server
            .join("Alice".into())
            .await
            .unwrap();

        server
            .begin_turn("Alice", 0)
            .await
            .unwrap();

        let result =
            server.end_turn("Alice").await.unwrap();

        assert_eq!(result.hope_gained, 0);
        assert_eq!(result.outcome, None);

        let state = server.snapshot().await;

        assert_eq!(state.turn.phase, TurnPhase::Idle);
        assert!(state.turn.active_player_id.is_none());
        assert!(state.turn.chain.is_empty());
        assert!(state.turn.rolls.is_empty());
        assert!(state.turn.heat_snapshot.is_empty());
    }

    #[tokio::test]
    async fn rolling_moves_player_and_records_roll() {
        let server = GameServer::new();

        server.start_game().await;

        let player = server
            .join("Alice".into())
            .await
            .unwrap();

        let starting_position =
            player.position.clone().unwrap();

        server
            .begin_turn("Alice", 0)
            .await
            .unwrap();

        let state_before =
            server.snapshot().await;

        let target = state_before
            .hex_order
            .iter()
            .find(|id| {
                let current =
                    state_before.hexes[&starting_position].clone();

                let target =
                    state_before.hexes[*id].clone();

                *id != &starting_position
                    && are_neighbors(&current, &target)
            })
            .cloned()
            .expect("starting hex should have a neighbor");

        let result = server
            .roll_for_hex("Alice", &target)
            .await
            .unwrap();

        assert!((1..=6).contains(&result.d1));
        assert!((1..=6).contains(&result.d2));

        let state =
            server.snapshot().await;

        assert_eq!(
            state.players["Alice"].position.as_deref(),
            Some(target.as_str())
        );

        assert_eq!(state.turn.chain, vec![target]);
        assert_eq!(state.turn.rolls.len(), 1);
    }

    #[tokio::test]
    async fn cannot_roll_non_adjacent_hex() {
        let server = GameServer::new();

        server.start_game().await;

        let player = server
            .join("Alice".into())
            .await
            .unwrap();

        server
            .begin_turn("Alice", 0)
            .await
            .unwrap();

        let state =
            server.snapshot().await;

        let start =
            player.position.unwrap();

        let distant = state
            .hex_order
            .iter()
            .find(|id| {
                let a = &state.hexes[&start];
                let b = &state.hexes[*id];

                crate::game::hex_distance(a, b) > 1
            })
            .cloned()
            .expect("board should contain a distant hex");

        let result = server
            .roll_for_hex("Alice", &distant)
            .await;

        assert_eq!(
            result,
            Err(GameError::HexNotAdjacent)
        );
    }
}