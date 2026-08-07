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

export interface RollResult {
  d1: number
  d2: number
  bonus: number
  heat: number
  modifier: number
  total: number
  outcome: Outcome
  critical: CriticalType
}

export type TurnPhase = 'idle' | 'active'

export interface TurnState {
  phase: TurnPhase
  activePlayerId: string | null
  bonus: number
  chain: HexId[]
  rolls: RollResult[]
  finalOutcome: Outcome | null
  heatSnapshot: Record<HexId, number>
}

export type MusicTrackId = 'none' | 'ambient' | 'forest' | 'mystic'

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
