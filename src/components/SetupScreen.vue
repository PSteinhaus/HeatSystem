<script setup lang="ts">
import { computed, ref } from 'vue'
import { setHexCount, startGame } from '../store'
import { DEFAULT_HEX_COUNT } from '../game/constants'

const count = ref(DEFAULT_HEX_COUNT)

const hexPreview = computed(() => {
  // simple preview count
  return count.value
})

function start() {
  setHexCount(count.value)
  startGame()
}
</script>

<template>
  <div class="setup-screen">
    <div class="setup-card">
      <h1 class="title">Hex & Orakel</h1>
      <p class="subtitle">Ein kooperatives Erzählspiel ohne Spielleitung</p>

      <div class="hex-control">
        <label>Anzahl Hexagonfelder</label>
        <div class="count-row">
          <button class="step-btn" @click="count = Math.max(7, count - 1)">−</button>
          <input
            type="number"
            v-model.number="count"
            :min="7"
            :max="60"
          />
          <button class="step-btn" @click="count = Math.min(60, count + 1)">+</button>
        </div>
        <p class="hint">Das Spielbrett besteht aus {{ hexPreview }} eng gepackten Hexagonen.</p>
      </div>

      <button class="primary start-btn" @click="start">Spiel starten</button>
    </div>
  </div>
</template>

<style scoped>
.setup-screen {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}
.setup-card {
  background: var(--bg-panel);
  border: 1px solid var(--border-light);
  border-radius: 20px;
  padding: 2.5rem;
  max-width: 420px;
  width: 100%;
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  animation: fadeIn 0.4s ease;
}
.title {
  font-size: 2rem;
  font-weight: 700;
  margin: 0 0 0.5rem;
  color: var(--primary);
  letter-spacing: 0.02em;
}
.subtitle {
  color: var(--text-muted);
  margin: 0 0 2rem;
  font-size: 0.95rem;
}
.hex-control {
  margin-bottom: 2rem;
}
.hex-control label {
  display: block;
  margin-bottom: 0.75rem;
  color: var(--text-muted);
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.count-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}
.count-row input {
  width: 80px;
  text-align: center;
  font-size: 1.2rem;
  font-weight: 600;
}
.step-btn {
  width: 40px;
  height: 40px;
  padding: 0;
  font-size: 1.3rem;
  display: flex;
  align-items: center;
  justify-content: center;
}
.hint {
  margin-top: 0.75rem;
  color: var(--text-dim);
  font-size: 0.85rem;
}
.start-btn {
  width: 100%;
  padding: 0.8em;
  font-size: 1rem;
}
</style>
