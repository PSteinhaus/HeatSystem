use std::collections::HashMap;

use crate::models::{
    GameState,
    GameStatus,
    Hex,
    MusicTrackId,
    TurnPhase,
    TurnState,
};

pub const DEFAULT_HEX_COUNT: usize = 19;
pub const STARTING_HOPE: i32 = 5;
pub const MAX_HEAT: i32 = 5;

pub const PLAYER_COLORS: &[&str] = &[
    "#e57373",
    "#64b5f6",
    "#81c784",
    "#ba68c8",
    "#ffb74d",
    "#4dd0e1",
];

pub fn generate_hexes(count: usize) -> Vec<Hex> {
    let mut result = Vec::with_capacity(count);

    /*
     * Start with the origin.
     */
    if count == 0 {
        return result;
    }

    result.push(Hex {
        id: hex_id(0, 0),
        q: 0,
        r: 0,
        heat: 0,
        inscription: String::new(),
    });

    if count == 1 {
        return result;
    }

    /*
     * Generate rings around the origin.
     *
     * This is only the initial server implementation.
     * We should replace this with the exact algorithm from
     * the existing client-side generateHexes().
     */
    let directions = [
        (1, 0),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (0, -1),
        (1, -1),
    ];

    let mut radius = 1;

    while result.len() < count {
        let (mut q, mut r) = (
            directions[4].0 * radius,
            directions[4].1 * radius,
        );

        for &(dq, dr) in &directions {
            for _ in 0..radius {
                if result.len() >= count {
                    return result;
                }

                result.push(Hex {
                    id: hex_id(q, r),
                    q,
                    r,
                    heat: 0,
                    inscription: String::new(),
                });

                q += dq;
                r += dr;
            }
        }

        radius += 1;
    }

    result
}

fn hex_id(q: i32, r: i32) -> String {
    format!("q{}-r{}", q, r)
}

pub fn default_game_state() -> GameState {
    let hexes_vec = generate_hexes(DEFAULT_HEX_COUNT);

    let mut hexes = HashMap::new();
    let mut hex_order = Vec::with_capacity(hexes_vec.len());

    for hex in hexes_vec {
        hex_order.push(hex.id.clone());
        hexes.insert(hex.id.clone(), hex);
    }

    GameState {
        status: GameStatus::Setup,
        hexes,
        hex_order,
        players: HashMap::new(),
        turn: TurnState {
            phase: TurnPhase::Idle,
            active_player_id: None,
            bonus: 0,
            chain: Vec::new(),
            rolls: Vec::new(),
            final_outcome: None,
            heat_snapshot: HashMap::new(),
        },
        music_track: MusicTrackId::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_expected_number_of_hexes() {
        let state = default_game_state();

        assert_eq!(state.hexes.len(), DEFAULT_HEX_COUNT);
        assert_eq!(state.hex_order.len(), DEFAULT_HEX_COUNT);
    }

    #[test]
    fn default_state_starts_in_setup() {
        let state = default_game_state();

        assert!(matches!(state.status, GameStatus::Setup));
        assert!(state.players.is_empty());
        assert!(matches!(state.turn.phase, TurnPhase::Idle));
    }

    #[test]
    fn generated_hex_ids_are_unique() {
        let hexes = generate_hexes(DEFAULT_HEX_COUNT);

        let ids: std::collections::HashSet<_> =
            hexes.iter().map(|h| h.id.clone()).collect();

        assert_eq!(ids.len(), hexes.len());
    }
}