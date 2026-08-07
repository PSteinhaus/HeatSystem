<script setup lang="ts">
import { computed } from 'vue'
import { gameState, playerList, myPlayer, isMyTurn, adjustHope, endGame, removePlayer } from '../store'

const emit = defineEmits<{ (e: 'openTurnModal'): void }>()

const activePlayerName = computed(() => {
  const id = gameState.turn.activePlayerId
  return id ? gameState.players[id]?.name : null
})

function onEndGame() {
  if (confirm('Spiel beenden? Alle Werte werden gelöscht.')) {
    endGame()
  }
}

function onRemovePlayer(playerId: string, name: string) {
  if (confirm(`Spieler „${name}" entfernen?`)) {
    removePlayer(playerId)
  }
}
</script>

<template>
  <div class="player-bar">
    <div class="players-section">
      <div class="section-label">Spieler</div>
      <div class="players-list">
        <div
          v-for="player in playerList"
          :key="player.id"
          class="player-chip"
          :class="{ active: gameState.turn.activePlayerId === player.id, me: player.id === myPlayer?.id }"
        >
          <span class="player-color" :style="{ background: player.color }"></span>
          <span class="player-name">{{ player.name }}</span>
          <button
            class="remove-player-btn"
            title="Spieler entfernen"
            @click.stop="onRemovePlayer(player.id, player.name)"
          >×</button>
          <span class="player-hope">
            <span class="hope-icon">✦</span>
            {{ player.hope }}
          </span>
          <div v-if="player.id === myPlayer?.id && gameState.turn.phase !== 'active'" class="hope-adjust">
            <button class="tiny-btn" @click="adjustHope(player.id, -1)">−</button>
            <button class="tiny-btn" @click="adjustHope(player.id, 1)">+</button>
          </div>
        </div>
      </div>
    </div>

    <div class="actions-section">
      <div v-if="gameState.turn.phase === 'active' && activePlayerName" class="turn-indicator">
        <span class="turn-dot"></span>
        {{ isMyTurn ? 'Du bist am Zug' : `${activePlayerName} ist am Zug` }}
      </div>
      <button
        v-if="myPlayer && gameState.turn.phase !== 'active'"
        class="primary begin-turn-btn"
        @click="emit('openTurnModal')"
      >Zug beginnen</button>
      <button class="end-game-btn" @click="onEndGame">Spiel beenden</button>
    </div>
  </div>
</template>

<style scoped>
.player-bar {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 1rem;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
}
.players-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.section-label {
  font-size: 0.7rem;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.players-list {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.player-chip {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 0.3em 0.7em;
  font-size: 0.85rem;
  transition: all 0.2s;
}
.player-chip.active {
  border-color: var(--primary);
  box-shadow: 0 0 0 1px var(--primary), 0 0 12px rgba(212, 166, 74, 0.3);
}
.player-chip.me {
  border-color: var(--accent);
}
.player-color {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.3);
}
.player-name {
  font-weight: 600;
}
.remove-player-btn {
  width: 18px;
  height: 18px;
  padding: 0;
  font-size: 0.85rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-dim);
  cursor: pointer;
  transition: all 0.2s;
  opacity: 0;
}
.player-chip:hover .remove-player-btn {
  opacity: 1;
}
.remove-player-btn:hover {
  background: var(--error);
  border-color: var(--error);
  color: #fff;
}
.player-hope {
  color: var(--primary);
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.2rem;
}
.hope-icon {
  font-size: 0.75em;
}
.hope-adjust {
  display: flex;
  gap: 0.15rem;
  margin-left: 0.2rem;
}
.tiny-btn {
  width: 22px;
  height: 22px;
  padding: 0;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
}
.actions-section {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.turn-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--primary);
  font-size: 0.9rem;
  font-weight: 600;
  padding: 0.4em 0.9em;
  background: rgba(212, 166, 74, 0.1);
  border-radius: 999px;
  border: 1px solid var(--primary-dim);
}
.turn-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--primary);
  animation: pulse-dot 1.2s ease-in-out infinite;
}
@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.8); }
}
.begin-turn-btn {
  font-size: 0.95rem;
  padding: 0.6em 1.5em;
}
.end-game-btn {
  font-size: 0.8rem;
  padding: 0.5em 1em;
  opacity: 0.6;
}
.end-game-btn:hover {
  opacity: 1;
}
</style>
