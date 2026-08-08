import type { GameState } from '../types'
import { generateHexes } from '../game/hex'
import { DEFAULT_HEX_COUNT } from '../game/constants'

const STORAGE_KEY = 'hex-oracle-game-state'
const CHANNEL_NAME = 'hex-oracle-sync'

function defaultState(): GameState {
  const hexes = generateHexes(DEFAULT_HEX_COUNT)
  const hexRecord: GameState['hexes'] = {}
  const hexOrder: string[] = []
  for (const h of hexes) {
    hexRecord[h.id] = h
    hexOrder.push(h.id)
  }
  return {
    status: 'setup',
    hexes: hexRecord,
    hexOrder,
    players: {},
    turn: {
      phase: 'idle',
      activePlayerId: null,
      bonus: 0,
      chain: [],
      rolls: [],
      finalOutcome: null,
      heatSnapshot: {},
      lastTurn: null,
    },
    musicTrack: 'none',
  }
}

type Listener = (state: GameState) => void

/**
 * Mock server for multiplayer synchronization.
 * Synchronizes the complete game state including:
 * - Board state (hexes, heat values, inscriptions)
 * - Player positions and hope
 * - Complete turn flow (phase, active player, rolls, chain, outcomes)
 * Uses BroadcastChannel for cross-tab communication and localStorage for persistence.
 */
class MockServer {
  private state: GameState
  private channel: BroadcastChannel | null
  private listeners = new Set<Listener>()

  constructor() {
    this.channel = typeof BroadcastChannel !== 'undefined' ? new BroadcastChannel(CHANNEL_NAME) : null
    this.state = this.load()
    if (this.channel) {
      this.channel.onmessage = (e: MessageEvent) => {
        this.state = e.data
        this.save()
        this.notify()
      }
    }
    window.addEventListener('storage', (e) => {
      if (e.key === STORAGE_KEY && e.newValue) {
        this.state = JSON.parse(e.newValue)
        this.notify()
      }
    })
  }

  private load(): GameState {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      try {
        return JSON.parse(stored)
      } catch {
        // fall through
      }
    }
    return defaultState()
  }

  private save() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.state))
  }

  private notify() {
    for (const fn of this.listeners) fn(this.state)
  }

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn)
    fn(this.state)
    return () => { this.listeners.delete(fn) }
  }

  getState(): GameState {
    return this.state
  }

  update(mutator: (s: GameState) => void) {
    mutator(this.state)
    this.save()
    if (this.channel) this.channel.postMessage(this.state)
    this.notify()
  }

  reset() {
    this.update(() => {
      this.state = defaultState()
    })
  }
}

export const mockServer = new MockServer()
