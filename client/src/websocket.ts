import type {
  GameState,
  MusicTrackId,
  Player,
  RollResult,
  Outcome,
} from './types'

type ConnectionHandler = (connected: boolean) => void

export type ClientMessage =
  | {
      type: 'join'
      name: string
    }
  | {
      type: 'start_game'
    }
  | {
      type: 'reset_game'
    }
  | {
      type: 'begin_turn'
      player_id: string
      bonus: number
    }
  | {
      type: 'roll_hex'
      player_id: string
      hex_id: string
    }
  | {
      type: 'end_turn'
      player_id: string
    }
  | {
      type: 'set_heat'
      hex_id: string
      heat: number
    }
  | {
      type: 'swap_hex_heat'
      hex_a: string
      hex_b: string
    }
  | {
      type: 'set_hope'
      player_id: string
      hope: number
    }
  | {
      type: 'adjust_hope'
      player_id: string
      delta: number
    }
  | {
      type: 'move_player'
      player_id: string
      hex_id: string
    }
  | {
      type: 'set_inscription'
      hex_id: string
      inscription: string
    }
  | {
      type: 'set_music_track'
      track: MusicTrackId
    }
  | {
      type: 'remove_player'
      player_id: string
    }
  | {
    type: 'set_hex_count'
    count: number
  }

export type ServerMessage =
  | {
      type: 'state'
      state: GameState
    }
  | {
      type: 'joined'
      player: Player
    }
  | {
      type: 'roll_result'
      result: RollResult
    }
  | {
      type: 'turn_ended'
      player_id: string
      hope_gained: number
      outcome: Outcome | null
    }
  | {
      type: 'error'
      message: string
    }

type MessageHandler = (message: ServerMessage) => void

export class GameWebSocket {
  private socket: WebSocket | null = null
  private handlers = new Set<MessageHandler>()
  private connectionHandlers = new Set<ConnectionHandler>()

  private reconnectTimer: number | null = null
  private manuallyClosed = false

  private reconnectDelay = 1000
  private readonly maxReconnectDelay = 10000

  subscribeConnection(
  handler: ConnectionHandler,
    ) {
    this.connectionHandlers.add(handler)

    handler(this.connected)

    return () => {
        this.connectionHandlers.delete(handler)
    }
    }

    private notifyConnection(
    connected: boolean,
    ) {
    for (const handler of this.connectionHandlers) {
        handler(connected)
    }
    }

  connect() {
    this.manuallyClosed = false

    if (
      this.socket &&
      (
        this.socket.readyState === WebSocket.OPEN ||
        this.socket.readyState === WebSocket.CONNECTING
      )
    ) {
      return
    }

    const protocol =
      window.location.protocol === 'https:' ? 'wss:' : 'ws:'

    const url = `${protocol}//${window.location.host}/ws`
    // FIXME: DEBUG
    const WS_URL = 'ws://127.0.0.1:8000/api/ws'

    const socket = new WebSocket(WS_URL)

    this.socket = socket

    socket.addEventListener('open', () => {
        this.reconnectDelay = 1000

        this.notifyConnection(true)
    })

    socket.addEventListener('message', (event) => {
      try {
        const message = JSON.parse(
          event.data,
        ) as ServerMessage

        this.notify(message)
      } catch (error) {
        console.error(
          'Failed to parse WebSocket message:',
          error,
        )
      }
    })

    socket.addEventListener('close', () => {
        if (this.socket === socket) {
            this.socket = null
        }

        this.notifyConnection(false)

        if (!this.manuallyClosed) {
            this.scheduleReconnect()
        }
    })

    socket.addEventListener('error', (error) => {
      console.error(
        'WebSocket error:',
        error,
      )
    })
  }

  disconnect() {
    this.manuallyClosed = true

    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    this.socket?.close()
    this.socket = null
  }

  subscribe(handler: MessageHandler) {
    this.handlers.add(handler)

    return () => {
      this.handlers.delete(handler)
    }
  }

  send(message: ClientMessage): boolean {
    if (
      !this.socket ||
      this.socket.readyState !== WebSocket.OPEN
    ) {
      console.warn(
        'Cannot send WebSocket message: socket is not connected.',
        message,
      )

      return false
    }

    this.socket.send(JSON.stringify(message))

    return true
  }

  get connected(): boolean {
    return (
      this.socket?.readyState === WebSocket.OPEN
    )
  }

  private notify(message: ServerMessage) {
    for (const handler of this.handlers) {
      handler(message)
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer !== null) {
      return
    }

    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null

      this.connect()

      this.reconnectDelay = Math.min(
        this.reconnectDelay * 2,
        this.maxReconnectDelay,
      )
    }, this.reconnectDelay)
  }
}

export const gameSocket = new GameWebSocket()