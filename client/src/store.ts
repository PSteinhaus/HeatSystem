import { reactive, computed, ref } from 'vue'

import type {
  GameState,
  HexId,
  Player,
  ToastMsg,
  Outcome,
  MusicTrackId,
  RollResult,
} from './types'

import {
  gameSocket,
  type ServerMessage,
} from './websocket'

type ServerGameState = {
  status: GameState['status']
  hexes: GameState['hexes']
  hex_order: HexId[]
  players: GameState['players']
  turn: {
    phase: GameState['turn']['phase']
    active_player_id: string | null
    bonus: number
    chain: HexId[]
    rolls: RollResult[]
    final_outcome: Outcome | null
    heat_snapshot: Record<HexId, number>
  }
  music_track: GameState['musicTrack']
}

export const gameState = reactive<GameState>({
  status: 'setup',

  hexes: {},
  hexOrder: [],

  players: {},

  turn: {
    phase: 'idle',
    activePlayerId: null,
    bonus: 0,
    chain: [],
    rolls: [],
    finalOutcome: null,
    heatSnapshot: {},
  },

  musicTrack: 'none',
})

export const me = reactive<{
  id: string | null
}>({
  id: null,
})

export const lastRollResult = ref<RollResult | null>(null)

export const lastTurnEnd = ref<{
  hopeGained: number
  outcome: Outcome | null
  playerId: string | null
} | null>(null)

export const toasts = reactive<ToastMsg[]>([])

export const myPlayer = computed<Player | null>(() =>
  me.id
    ? gameState.players[me.id] ?? null
    : null,
)

export const activeTurn = computed(
  () => gameState.turn.phase === 'active',
)

export const isMyTurn = computed(
  () =>
    activeTurn.value &&
    gameState.turn.activePlayerId === me.id,
)

export const playerList = computed(
  () => Object.values(gameState.players),
)

export const hexList = computed(
  () =>
    gameState.hexOrder
      .map((id) => gameState.hexes[id])
      .filter(Boolean),
)

export const activePlayer = computed(() => {
  const id = gameState.turn.activePlayerId

  return id
    ? gameState.players[id] ?? null
    : null
})

export const currentTurnBonus = computed(
  () => gameState.turn.bonus,
)

export const currentChain = computed(
  () => gameState.turn.chain,
)

export const currentRolls = computed(
  () => gameState.turn.rolls,
)

export const turnFinalOutcome = computed(
  () => gameState.turn.finalOutcome,
)

export const turnHeatSnapshot = computed(
  () => gameState.turn.heatSnapshot,
)

export function isPlayerActive(
  playerId: string | null,
): boolean {
  return (
    activeTurn.value &&
    gameState.turn.activePlayerId === playerId
  )
}

// ---------------------------------------------------------------------------
// TOASTS
// ---------------------------------------------------------------------------

let toastCounter = 0

export function showToast(
  message: string,
  type: ToastMsg['type'] = 'info',
) {
  const id = ++toastCounter

  toasts.push({
    id,
    message,
    type,
  })

  setTimeout(() => {
    const idx = toasts.findIndex(
      (t) => t.id === id,
    )

    if (idx >= 0) {
      toasts.splice(idx, 1)
    }
  }, 3500)
}

// ---------------------------------------------------------------------------
// SERVER STATE SYNCHRONIZATION
// ---------------------------------------------------------------------------

function syncState(
  target: GameState,
  source: ServerGameState,
) {
  target.status = source.status

  target.hexes = {}

  for (const [key, value] of Object.entries(source.hexes)) {
    target.hexes[key] = {
      ...value,
    }
  }

  target.hexOrder = [
    ...source.hex_order,
  ]

  target.players = {}

  for (const [key, value] of Object.entries(source.players)) {
    target.players[key] = {
      ...value,
    }
  }

  target.turn = {
    phase: source.turn.phase,

    activePlayerId:
      source.turn.active_player_id,

    bonus: source.turn.bonus,

    chain: [
      ...source.turn.chain,
    ],

    rolls: [
      ...source.turn.rolls,
    ],

    finalOutcome:
      source.turn.final_outcome,

    heatSnapshot: {
      ...source.turn.heat_snapshot,
    },
  }

  target.musicTrack =
    source.music_track
}

function handleServerMessage(
  message: ServerMessage,
) {
  switch (message.type) {
    case 'state':
      syncState(
        gameState,
        message.state,
      )
      break

    case 'joined':
      /*
       * The server deliberately uses the player's name
       * as the player's stable ID.
       */
      me.id = message.player.id

      /*
       * The following State message will contain the
       * authoritative player object.
       */
      break

    case 'roll_result':
      lastRollResult.value = message.result
      break

    case 'turn_ended':
      lastTurnEnd.value = {
        hopeGained: message.hope_gained,
        outcome: message.outcome,
        playerId: gameState.turn.activePlayerId,
      }
      break

    case 'error':
      showToast(
        message.message,
        'failure',
      )
      break
  }
}

gameSocket.subscribe(handleServerMessage)

// Connect immediately when this module is loaded.
gameSocket.connect()

// ---------------------------------------------------------------------------
// CONNECTION
// ---------------------------------------------------------------------------

export function connectName(
  name: string,
): boolean {
  const trimmed = name.trim()

  if (!trimmed) {
    showToast(
      'Bitte gib einen Namen ein.',
      'failure',
    )

    return false
  }

  if (gameSocket.connected) {
    return gameSocket.send({
      type: 'join',
      name: trimmed,
    })
  }

  pendingJoin = trimmed

  gameSocket.connect()

  return true
}

// ---------------------------------------------------------------------------
// JOIN QUEUE
// ---------------------------------------------------------------------------

let pendingJoin: string | null = null

gameSocket.subscribeConnection((connected) => {
  if (!connected || !pendingJoin) {
    return
  }

  const name = pendingJoin
  pendingJoin = null

  gameSocket.send({
    type: 'join',
    name,
  })
})

// ---------------------------------------------------------------------------
// GAME LIFECYCLE
// ---------------------------------------------------------------------------

export function setHexCount(count: number) {
  gameSocket.send({
    type: 'set_hex_count',
    count,
  })
}

export function startGame() {
  gameSocket.send({
    type: 'start_game',
  })
}

export function endGame() {
  /*
   * The Rust server calls this operation reset_game.
   */
  gameSocket.send({
    type: 'reset_game',
  })

  me.id = null
}

// ---------------------------------------------------------------------------
// BOARD / ADMINISTRATION
// ---------------------------------------------------------------------------

export function setHeat(
  hexId: HexId,
  heat: number,
) {
  gameSocket.send({
    type: 'set_heat',
    hex_id: hexId,
    heat,
  })
}

export function movePlayer(
  playerId: string,
  hexId: HexId,
) {
  gameSocket.send({
    type: 'move_player',
    player_id: playerId,
    hex_id: hexId,
  })
}

export function swapHexHeat(
  hexA: HexId,
  hexB: HexId,
) {
  gameSocket.send({
    type: 'swap_hex_heat',
    hex_a: hexA,
    hex_b: hexB,
  })
}

export function setHope(
  playerId: string,
  hope: number,
) {
  gameSocket.send({
    type: 'set_hope',
    player_id: playerId,
    hope,
  })
}

export function adjustHope(
  playerId: string,
  delta: number,
) {
  gameSocket.send({
    type: 'adjust_hope',
    player_id: playerId,
    delta,
  })
}

export function setInscription(
  hexId: HexId,
  inscription: string,
) {
  gameSocket.send({
    type: 'set_inscription',
    hex_id: hexId,
    inscription,
  })
}

export function setMusicTrack(
  track: MusicTrackId,
) {
  gameSocket.send({
    type: 'set_music_track',
    track,
  })
}

export function removePlayer(
  playerId: string,
) {
  gameSocket.send({
    type: 'remove_player',
    player_id: playerId,
  })
}

// ---------------------------------------------------------------------------
// TURN MANAGEMENT
// ---------------------------------------------------------------------------

export function beginTurn(bonus: number): boolean {
  if (!me.id) {
    return false
  }

  return gameSocket.send({
    type: 'begin_turn',
    player_id: me.id,
    bonus,
  })
}

export function rollForHex(hexId: HexId): boolean {
  if (!me.id) {
    return false
  }

  return gameSocket.send({
    type: 'roll_hex',
    player_id: me.id,
    hex_id: hexId,
  })
}

export function endTurn(): boolean {
  if (!me.id) {
    return false
  }

  return gameSocket.send({
    type: 'end_turn',
    player_id: me.id,
  })
}

export function endTurnAuto(): boolean {
  return endTurn()
}