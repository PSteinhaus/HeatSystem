export type HexId = string

export interface Hex {
  id: HexId
  q: number
  r: number
  heat: number
  inscription: string
}

export interface Player {
  id: string
  name: string
  hope: number
  color: string
  position: HexId | null
}

export type Outcome = 'success' | 'mixed' | 'failure'
export type CriticalType = 'critical_success' | 'critical_failure' | 'upgrade' | null

/** Result of a single dice roll - synchronized across all players */
export interface RollResult {
  /** First die value (1-6) */
  d1: number
  /** Second die value (1-6) */
  d2: number
  /** Bonus used for this roll */
  bonus: number
  /** Heat value of the hex being rolled */
  heat: number
  /** Modifier applied (heat + bonus) */
  modifier: number
  /** Total roll value (d1 + d2 + modifier) */
  total: number
  /** Outcome of the roll */
  outcome: Outcome
  /** Critical type if applicable */
  critical: CriticalType
}

/** Turn phases for multiplayer synchronization */
export type TurnPhase = 'idle' | 'active'

/** Complete turn state - synchronized across all players */
export interface TurnState {
  /** Current phase: 'idle' when no turn is active, 'active' when a player is taking their turn */
  phase: TurnPhase
  /** ID of the player currently taking their turn, or null if no turn is active */
  activePlayerId: string | null
  /** Bonus value used for this turn (0-3) */
  bonus: number
  /** Hexes visited during this turn in order */
  chain: HexId[]
  /** All dice rolls made during this turn */
  rolls: RollResult[]
  /** Final outcome if turn ended with failure, otherwise null */
  finalOutcome: Outcome | null
  /** Snapshot of heat values at the start of the turn for hope calculation */
  heatSnapshot: Record<HexId, number>
}

export type MusicTrackId = 'none' | 'ambient' | 'forest' | 'mystic'

/** Complete game state - fully synchronized across all players via mockServer */
export interface GameState {
  status: 'setup' | 'playing' | 'ended'
  hexes: Record<HexId, Hex>
  hexOrder: HexId[]
  players: Record<string, Player>
  turn: TurnState
  musicTrack: MusicTrackId
}

export interface ToastMsg {
  id: number
  message: string
  type: 'info' | 'success' | 'failure'
}
