import { reactive, computed } from 'vue'
import type { GameState, HexId, Player, ToastMsg, Outcome, MusicTrackId, RollResult } from './types'
import { mockServer } from './server/mockServer'
import { generateHexes, areNeighbors } from './game/hex'
import { PLAYER_COLORS, STARTING_HOPE, MAX_HEAT, DEFAULT_HEX_COUNT } from './game/constants'
import { bonusCost, calculateOutcome, calculateHopeGained } from './game/logic'

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

export const me = reactive<{ id: string | null }>({ id: null })

export const toasts = reactive<ToastMsg[]>([])

export const myPlayer = computed<Player | null>(() => (me.id ? gameState.players[me.id] ?? null : null))
export const activeTurn = computed(() => gameState.turn.phase === 'active')
export const isMyTurn = computed(() => activeTurn.value && gameState.turn.activePlayerId === me.id)
export const playerList = computed(() => Object.values(gameState.players))
export const hexList = computed(() => gameState.hexOrder.map((id) => gameState.hexes[id]).filter(Boolean))

// Turn flow synchronization - these allow all players to see the current turn state
export const activePlayer = computed(() => {
  const id = gameState.turn.activePlayerId
  return id ? gameState.players[id] ?? null : null
})
export const currentTurnBonus = computed(() => gameState.turn.bonus)
export const currentChain = computed(() => gameState.turn.chain)
export const currentRolls = computed(() => gameState.turn.rolls)
export const turnFinalOutcome = computed(() => gameState.turn.finalOutcome)
export const turnHeatSnapshot = computed(() => gameState.turn.heatSnapshot)

// Check if a specific player is the active player
export function isPlayerActive(playerId: string | null): boolean {
  return activeTurn.value && gameState.turn.activePlayerId === playerId
}

let toastCounter = 0
export function showToast(message: string, type: ToastMsg['type'] = 'info') {
  const id = ++toastCounter
  toasts.push({ id, message, type })
  setTimeout(() => {
    const idx = toasts.findIndex((t) => t.id === id)
    if (idx >= 0) toasts.splice(idx, 1)
  }, 3500)
}

function syncState(target: GameState, source: GameState) {
  // Deep sync game state from source to target
  target.status = source.status
  
  // Sync hexes
  target.hexes = {}
  for (const [key, val] of Object.entries(source.hexes)) {
    target.hexes[key] = { ...val }
  }
  
  // Sync hexOrder
  target.hexOrder = [...source.hexOrder]
  
  // Sync players
  target.players = {}
  for (const [key, val] of Object.entries(source.players)) {
    target.players[key] = { ...val }
  }
  
  // Sync turn state - this ensures all turn flow data is synchronized
  target.turn = {
    phase: source.turn.phase,
    activePlayerId: source.turn.activePlayerId,
    bonus: source.turn.bonus,
    chain: [...source.turn.chain],
    rolls: [...source.turn.rolls],
    finalOutcome: source.turn.finalOutcome,
    heatSnapshot: { ...source.turn.heatSnapshot },
  }
  
  target.musicTrack = source.musicTrack
}

mockServer.subscribe((state) => {
  syncState(gameState, state)
})

export function setHexCount(count: number) {
  if (gameState.status !== 'setup') return
  mockServer.update((s) => {
    const hexes = generateHexes(count)
    s.hexes = {}
    s.hexOrder = []
    for (const h of hexes) {
      s.hexes[h.id] = h
      s.hexOrder.push(h.id)
    }
  })
}

export function startGame() {
  mockServer.update((s) => {
    s.status = 'playing'
    s.players = {}
    s.musicTrack = 'none'
    s.turn = {
      phase: 'idle',
      activePlayerId: null,
      bonus: 0,
      chain: [],
      rolls: [],
      finalOutcome: null,
      heatSnapshot: {},
    }
  })
}

export function endGame() {
  mockServer.update((s) => {
    s.status = 'setup'
    s.players = {}
    s.musicTrack = 'none'
    s.turn = {
      phase: 'idle',
      activePlayerId: null,
      bonus: 0,
      chain: [],
      rolls: [],
      finalOutcome: null,
      heatSnapshot: {},
    }
    const hexes = generateHexes(DEFAULT_HEX_COUNT)
    s.hexes = {}
    s.hexOrder = []
    for (const h of hexes) {
      s.hexes[h.id] = h
      s.hexOrder.push(h.id)
    }
  })
  me.id = null
}

export function connectName(name: string): boolean {
  if (gameState.status !== 'playing') return false
  const existing = gameState.players[name]
  if (existing) {
    me.id = name
    return true
  }
  const colorIdx = Object.keys(gameState.players).length % PLAYER_COLORS.length
  const occupied = new Set(Object.values(gameState.players).map((p) => p.position))
  const freeHex = gameState.hexOrder.find((id) => !occupied.has(id))
  mockServer.update((s) => {
    s.players[name] = {
      id: name,
      name,
      hope: STARTING_HOPE,
      color: PLAYER_COLORS[colorIdx],
      position: freeHex ?? null,
    }
  })
  me.id = name
  return true
}

export function setHeat(hexId: HexId, heat: number) {
  if (activeTurn.value) return
  mockServer.update((s) => {
    if (s.hexes[hexId]) s.hexes[hexId].heat = Math.min(MAX_HEAT, Math.max(-MAX_HEAT, heat))
  })
}

export function movePlayer(playerId: string, hexId: HexId) {
  if (activeTurn.value && playerId !== gameState.turn.activePlayerId) return
  mockServer.update((s) => {
    if (s.players[playerId] && s.hexes[hexId]) s.players[playerId].position = hexId
  })
}

export function swapHexHeat(hexA: HexId, hexB: HexId) {
  if (activeTurn.value) return
  mockServer.update((s) => {
    const a = s.hexes[hexA]
    const b = s.hexes[hexB]
    if (a && b) {
      const tmp = a.heat
      a.heat = b.heat
      b.heat = tmp
    }
  })
}

export function setHope(playerId: string, hope: number) {
  if (activeTurn.value) return
  mockServer.update((s) => {
    if (s.players[playerId]) s.players[playerId].hope = Math.max(0, hope)
  })
}

export function adjustHope(playerId: string, delta: number) {
  if (activeTurn.value) return
  mockServer.update((s) => {
    if (s.players[playerId]) s.players[playerId].hope = Math.max(0, s.players[playerId].hope + delta)
  })
}

export function beginTurn(bonus: number) {
  if (activeTurn.value || !me.id) return
  const player = gameState.players[me.id]
  if (!player || !player.position) return
  const cost = bonusCost(bonus)
  if (player.hope < cost) return
  mockServer.update((s) => {
    const p = s.players[me.id!]
    if (!p) return
    p.hope -= cost
    s.turn = {
      phase: 'active',
      activePlayerId: me.id!,
      bonus,
      chain: [],
      rolls: [],
      finalOutcome: null,
      heatSnapshot: Object.fromEntries(
        Object.values(s.hexes).map((h) => [h.id, h.heat])
      ),
    }
  })
}

export function rollForHex(hexId: HexId): RollResult | null {
  if (!isMyTurn.value || !me.id) return null
  const player = gameState.players[me.id]
  if (!player) return null
  const targetHex = gameState.hexes[hexId]
  if (!targetHex) return null

  const currentPos = gameState.turn.chain.length > 0
    ? gameState.turn.chain[gameState.turn.chain.length - 1]
    : player.position
  if (!currentPos) return null
  const currentHex = gameState.hexes[currentPos]
  if (!currentHex || !areNeighbors(currentHex, targetHex)) return null
  if (gameState.turn.chain.includes(hexId)) return null

  const isContinuation = gameState.turn.rolls.length > 0
  const cost = bonusCost(gameState.turn.bonus)
  if (isContinuation && player.hope < cost) {
    showToast('Nicht genug Hoffnung, um den Bonus erneut zu zahlen.', 'failure')
    return null
  }

  // Generate dice values - only the active player does this, and it will be
  // synchronized to all other players via the mockServer update
  const d1 = Math.floor(Math.random() * 6) + 1
  const d2 = Math.floor(Math.random() * 6) + 1
  const result = calculateOutcome(d1, d2, gameState.turn.bonus, targetHex.heat)

  mockServer.update((s) => {
    const p = s.players[me.id!]
    if (!p) return
    if (isContinuation) p.hope -= cost
    p.position = hexId
    s.turn.chain.push(hexId)
    s.turn.rolls.push(result)
    if (result.outcome === 'failure') {
      s.turn.finalOutcome = 'failure'
    }
  })

  return result
}

export function endTurn(): { hopeGained: number; outcome: Outcome | null } | null {
  if (!isMyTurn.value || !me.id) return null
  const turn = gameState.turn
  const lastOutcome = turn.rolls.length > 0 ? turn.rolls[turn.rolls.length - 1].outcome : null
  const chainHeatSum = turn.chain.reduce((sum, hexId) => sum + Math.max(0, turn.heatSnapshot[hexId] ?? 0), 0)
  const hopeGained = lastOutcome ? calculateHopeGained(lastOutcome, chainHeatSum) : 0

  mockServer.update((s) => {
    if (me.id && s.players[me.id]) {
      s.players[me.id].hope += hopeGained
    }
    s.turn = {
      phase: 'idle',
      activePlayerId: null,
      bonus: 0,
      chain: [],
      rolls: [],
      finalOutcome: null,
      heatSnapshot: {},
    }
  })
  return { hopeGained, outcome: lastOutcome }
}

export function endTurnAuto(): { hopeGained: number; outcome: Outcome | null } | null {
  return endTurn()
}

export function setMusicTrack(track: MusicTrackId) {
  mockServer.update((s) => {
    s.musicTrack = track
  })
}

export function setInscription(hexId: HexId, inscription: string) {
  mockServer.update((s) => {
    if (s.hexes[hexId]) s.hexes[hexId].inscription = inscription.slice(0, 100)
  })
}

export function removePlayer(playerId: string) {
  mockServer.update((s) => {
    delete s.players[playerId]
  })
  if (me.id === playerId) me.id = null
}
