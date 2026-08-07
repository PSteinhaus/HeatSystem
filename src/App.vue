<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import SetupScreen from './components/SetupScreen.vue'
import NamePrompt from './components/NamePrompt.vue'
import HexBoard from './components/HexBoard.vue'
import PlayerBar from './components/PlayerBar.vue'
import TurnModal from './components/TurnModal.vue'
import DiceRoller from './components/DiceRoller.vue'
import MusicPlayer from './components/MusicPlayer.vue'
import ToastHost from './components/ToastHost.vue'
import { gameState, me, isMyTurn, endTurn, showToast } from './store'
import { OUTCOME_LABELS, CRITICAL_LABELS } from './game/logic'
import type { Outcome } from './types'

const showTurnModal = ref(false)
const showNamePrompt = ref(false)
const extendMode = ref(false)

// dice display
const diceTrigger = ref(0)
const diceValues = ref({ d1: 1, d2: 1 })
const showDice = ref(false)
const lastRoll = ref<{ outcome: Outcome; total: number; modifier: number; critical: string | null } | null>(null)

function startExtendMode() {
  extendMode.value = true
}

function endExtendMode() {
  extendMode.value = false
}

// outcome tint
const outcomeTint = ref<'success' | 'mixed' | 'failure' | null>(null)

const showHopeGained = ref<{ amount: number; outcome: Outcome } | null>(null)

const showSetup = computed(() => gameState.status === 'setup')
const showGame = computed(() => gameState.status === 'playing')

onMounted(() => {
  // check if we need name prompt
  if (gameState.status === 'playing' && !me.id) {
    showNamePrompt.value = true
  }
})

watch(() => gameState.status, (status) => {
  if (status === 'playing' && !me.id) {
    showNamePrompt.value = true
  } else if (status === 'setup') {
    showNamePrompt.value = false
    me.id = null
  }
})

// Watch for new rolls in the turn
watch(() => gameState.turn.rolls.length, (newLen, oldLen) => {
  if (newLen > (oldLen ?? 0)) {
    const lastRollResult = gameState.turn.rolls[newLen - 1]
    // Clear previous roll result immediately so old result doesn't show
    lastRoll.value = null
    // Set dice values and trigger in correct order for animation
    diceValues.value = { d1: lastRollResult.d1, d2: lastRollResult.d2 }
    showDice.value = true
    // Exit extend mode when a new roll happens so dice can show
    extendMode.value = false
    // Increment trigger after showing to ensure DiceRoller is mounted
    setTimeout(() => {
      diceTrigger.value++
    }, 10)

    setTimeout(() => {
      lastRoll.value = {
        outcome: lastRollResult.outcome,
        total: lastRollResult.total,
        modifier: lastRollResult.modifier,
        critical: lastRollResult.critical,
      }
      outcomeTint.value = lastRollResult.outcome

      const critLabel = lastRollResult.critical ? CRITICAL_LABELS[lastRollResult.critical] : null
      const msg = critLabel
        ? `${critLabel} — ${OUTCOME_LABELS[lastRollResult.outcome]} (Wurf: ${lastRollResult.total})`
        : `${OUTCOME_LABELS[lastRollResult.outcome]} (Wurf: ${lastRollResult.total})`
      showToast(msg, lastRollResult.outcome === 'success' ? 'success' : lastRollResult.outcome === 'failure' ? 'failure' : 'info')

      // Show end turn button for all outcomes including failure
      // No auto-end on failure - user must click "Zug beenden"
    }, 1400)
  }
})

// Watch for turn ending (phase goes back to idle) to handle manual end
watch(() => gameState.turn.phase, (phase, oldPhase) => {
  if (oldPhase === 'active' && phase === 'idle') {
    // Turn ended — if not already handled by auto-end, show hope gained
    // This handles the case where the active player ended manually
    // The hope was already added in endTurn()
  }
})

function onEndTurn() {
  const result = endTurn()
  if (result) {
    showHopeGained.value = { amount: result.hopeGained, outcome: result.outcome ?? 'mixed' }
    setTimeout(() => { showHopeGained.value = null }, 3000)
    window.setTimeout(() => {
      outcomeTint.value = null
      lastRoll.value = null
      showDice.value = false
    }, 4000)
  }
}

function closeNamePrompt() {
  showNamePrompt.value = false
}
</script>

<template>
  <div class="app-root">
    <SetupScreen v-if="showSetup" />

    <template v-if="showGame">
      <div class="game-layout">
        <PlayerBar @open-turn-modal="showTurnModal = true" />

        <div class="board-area">
          <HexBoard :outcome-tint="outcomeTint" :extend-mode="extendMode" @extend="startExtendMode" @end-extend="endExtendMode" />

          <MusicPlayer />

          <!-- Dice overlay -->
          <transition name="fade">
            <div v-if="showDice && !extendMode" class="dice-overlay">
              <DiceRoller
                :d1="diceValues.d1"
                :d2="diceValues.d2"
                :trigger="diceTrigger"
              />
              <div v-if="lastRoll" class="roll-result" :class="lastRoll.outcome">
                <div class="roll-total">{{ lastRoll.total }}</div>
                <div class="roll-detail">
                  Wurf + Modifikator ({{ lastRoll.modifier >= 0 ? '+' : '' }}{{ lastRoll.modifier }})
                </div>
                <div class="roll-outcome">{{ OUTCOME_LABELS[lastRoll.outcome] }}</div>
                <div v-if="lastRoll.critical" class="roll-critical">{{ CRITICAL_LABELS[lastRoll.critical] }}</div>
              </div>
            </div>
          </transition>

          <!-- Hope gained overlay -->
          <transition name="pop">
            <div v-if="showHopeGained" class="hope-overlay" :class="showHopeGained.outcome">
              <span class="hope-icon">✦</span>
              <span class="hope-amount">+{{ showHopeGained.amount }}</span>
              <span class="hope-label">Hoffnung</span>
            </div>
          </transition>

          <!-- End turn button (only for active player, shown for all outcomes) -->
          <div v-if="isMyTurn && lastRoll" class="end-turn-bar">
            <div class="end-turn-btns">
              <button class="primary end-turn-btn" @click="onEndTurn">Zug beenden</button>
              <button v-if="lastRoll.outcome !== 'failure'" class="secondary extend-btn" @click="startExtendMode">Zug erweitern</button>
            </div>
            <p v-if="lastRoll.outcome !== 'failure' && !extendMode" class="end-turn-hint">Oder bewege deine Figur auf ein Nachbarfeld um weiterzuwürfeln</p>
          </div>
        </div>
      </div>
    </template>

    <TurnModal :open="showTurnModal" @close="showTurnModal = false" />
    <NamePrompt v-if="showNamePrompt" @close="closeNamePrompt" />
    <ToastHost />
  </div>
</template>

<style scoped>
.app-root {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
.game-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.board-area {
  flex: 1;
  position: relative;
  overflow: hidden;
}
.dice-overlay {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  z-index: 50;
  pointer-events: none;
}
.roll-result {
  text-align: center;
  background: var(--bg-panel);
  border: 1px solid var(--border-light);
  border-radius: 12px;
  padding: 0.75rem 1.5rem;
  animation: fadeIn 0.3s ease;
}
.roll-result.success {
  border-color: var(--primary);
}
.roll-result.failure {
  border-color: var(--error);
}
.roll-result.mixed {
  border-color: var(--text-muted);
}
.roll-total {
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--text);
  line-height: 1;
}
.roll-detail {
  font-size: 0.8rem;
  color: var(--text-dim);
  margin-top: 0.25rem;
}
.roll-outcome {
  font-size: 1.1rem;
  font-weight: 600;
  margin-top: 0.5rem;
}
.roll-result.success .roll-outcome { color: var(--primary); }
.roll-result.failure .roll-outcome { color: var(--error); }
.roll-result.mixed .roll-outcome { color: var(--text-muted); }
.roll-critical {
  font-size: 0.9rem;
  font-weight: 700;
  margin-top: 0.25rem;
  color: var(--accent);
}
.hope-overlay {
  position: absolute;
  top: 30%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  z-index: 60;
  pointer-events: none;
  animation: popIn 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.hope-overlay.success { color: var(--primary); }
.hope-overlay.mixed { color: var(--text); }
.hope-overlay.failure { color: var(--text-muted); }
.hope-icon {
  font-size: 3rem;
  filter: drop-shadow(0 0 12px currentColor);
}
.hope-amount {
  font-size: 2.5rem;
  font-weight: 700;
}
.hope-label {
  font-size: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-muted);
}
.end-turn-bar {
  position: absolute;
  bottom: 1.5rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  z-index: 40;
  animation: fadeIn 0.3s ease;
}
.end-turn-btns {
  display: flex;
  gap: 0.75rem;
}
.end-turn-btn {
  font-size: 1.05rem;
  padding: 0.7em 2.5em;
  box-shadow: 0 4px 20px rgba(212, 166, 74, 0.3);
}
.extend-btn {
  font-size: 1.05rem;
  padding: 0.7em 2.5em;
}
.end-turn-hint {
  color: var(--text-dim);
  font-size: 0.8rem;
  margin: 0;
}
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.4s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
.pop-enter-active {
  animation: popIn 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.pop-leave-active {
  transition: opacity 0.4s ease;
}
.pop-leave-to {
  opacity: 0;
}
@keyframes popIn {
  from { opacity: 0; transform: translate(-50%, -50%) scale(0.5); }
  to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
}
</style>
