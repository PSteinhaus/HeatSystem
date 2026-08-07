<script setup lang="ts">
import { computed, ref, reactive, onMounted, onUnmounted } from 'vue'
import {
  gameState, isMyTurn, myPlayer, setHeat, movePlayer, swapHexHeat,
  rollForHex, hexList, playerList, setInscription,
} from '../store'
import {
  HEX_SIZE, hexToPixel, hexCorners, areNeighbors,
} from '../game/hex'
import { MAX_HEAT } from '../game/constants'
import type { HexId, Hex } from '../types'

const props = defineProps<{
  outcomeTint: 'success' | 'mixed' | 'failure' | null
}>()

const svgRef = ref<SVGSVGElement | null>(null)
const selectedHex = ref<HexId | null>(null)
const swapTarget = ref<HexId | null>(null)

const drag = reactive({
  active: false,
  playerId: null as string | null,
  x: 0,
  y: 0,
  startX: 0,
  startY: 0,
  hexId: null as HexId | null,
})

function screenToSvg(clientX: number, clientY: number): { x: number; y: number } {
  const svg = svgRef.value
  if (!svg) return { x: 0, y: 0 }
  const pt = svg.createSVGPoint()
  pt.x = clientX
  pt.y = clientY
  const ctm = svg.getScreenCTM()
  if (!ctm) return { x: 0, y: 0 }
  const inv = ctm.inverse()
  const result = pt.matrixTransform(inv)
  return { x: result.x, y: result.y }
}

function onFigurePointerDown(e: PointerEvent, playerId: string) {
  e.stopPropagation()
  const player = gameState.players[playerId]
  if (!player || !player.position) return

  // During turn, only active player can move their own figure
  if (isMyTurn.value && playerId !== myPlayer.value?.id) return
  if (gameState.turn.phase === 'active' && playerId !== gameState.turn.activePlayerId) return

  drag.active = true
  drag.playerId = playerId
  drag.hexId = player.position
  const svgPt = screenToSvg(e.clientX, e.clientY)
  drag.x = svgPt.x
  drag.y = svgPt.y
  drag.startX = svgPt.x
  drag.startY = svgPt.y
  ;(e.target as Element).setPointerCapture(e.pointerId)
}

function onPointerMove(e: PointerEvent) {
  if (!drag.active) return
  const svgPt = screenToSvg(e.clientX, e.clientY)
  drag.x = svgPt.x
  drag.y = svgPt.y
}

function onPointerUp(e: PointerEvent) {
  if (!drag.active) return
  const svgPt = screenToSvg(e.clientX, e.clientY)
  const targetHex = findHexAtPoint(svgPt.x, svgPt.y)
  if (targetHex && drag.playerId) {
    if (isMyTurn.value && drag.playerId === myPlayer.value?.id) {
      // Turn mode: must be neighbor, not in chain
      const currentChain = gameState.turn.chain
      const currentPos = currentChain.length > 0
        ? currentChain[currentChain.length - 1]
        : myPlayer.value?.position
      if (currentPos) {
        const currentHex = gameState.hexes[currentPos]
        if (currentHex && areNeighbors(currentHex, targetHex) && !currentChain.includes(targetHex.id)) {
          rollForHex(targetHex.id)
        }
      }
    } else if (gameState.turn.phase !== 'active') {
      // Free move
      movePlayer(drag.playerId, targetHex.id)
    }
  }
  drag.active = false
  drag.playerId = null
  drag.hexId = null
}

function findHexAtPoint(x: number, y: number): Hex | null {
  for (const hex of hexList.value) {
    const { x: hx, y: hy } = hexToPixel(hex.q, hex.r)
    const dx = x - hx
    const dy = y - hy
    // approximate with circle (radius = HEX_SIZE * 0.9)
    if (dx * dx + dy * dy < (HEX_SIZE * 0.85) ** 2) {
      return hex
    }
  }
  return null
}

function onHexClick(e: MouseEvent, hexId: HexId) {
  if (isMyTurn.value || gameState.turn.phase === 'active') return
  if (e.shiftKey || e.button === 2) {
    // right-click / shift-click: decrement heat
    setHeat(hexId, gameState.hexes[hexId].heat - 1)
  } else if (swapTarget.value && swapTarget.value !== hexId) {
    swapHexHeat(swapTarget.value, hexId)
    swapTarget.value = null
  } else {
    // left click: select
    if (selectedHex.value === hexId) {
      selectedHex.value = null
    } else {
      selectedHex.value = hexId
      swapTarget.value = null
    }
  }
}

function onHexContext(e: MouseEvent, hexId: HexId) {
  e.preventDefault()
  if (isMyTurn.value || gameState.turn.phase === 'active') return
  setHeat(hexId, gameState.hexes[hexId].heat - 1)
}

function incrementHeat(hexId: HexId) {
  setHeat(hexId, gameState.hexes[hexId].heat + 1)
}
function decrementHeat(hexId: HexId) {
  setHeat(hexId, gameState.hexes[hexId].heat - 1)
}

function onInscriptionInput(hexId: HexId, e: Event) {
  setInscription(hexId, (e.target as HTMLInputElement).value)
}

const viewBox = computed(() => {
  const hexes = hexList.value
  if (hexes.length === 0) return '-100 -100 200 200'
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity
  for (const hex of hexes) {
    const { x, y } = hexToPixel(hex.q, hex.r)
    minX = Math.min(minX, x)
    maxX = Math.max(maxX, x)
    minY = Math.min(minY, y)
    maxY = Math.max(maxY, y)
  }
  const pad = HEX_SIZE * 1.3
  return `${(minX - pad).toFixed(1)} ${(minY - pad).toFixed(1)} ${((maxX - minX) + pad * 2).toFixed(1)} ${((maxY - minY) + pad * 2).toFixed(1)}`
})

const validMoveTargets = computed<Set<HexId>>(() => {
  if (!isMyTurn.value || !myPlayer.value?.position) return new Set()
  const chain = gameState.turn.chain
  const currentPos = chain.length > 0 ? chain[chain.length - 1] : myPlayer.value.position
  const currentHex = gameState.hexes[currentPos]
  if (!currentHex) return new Set()
  const targets = new Set<HexId>()
  for (const hex of hexList.value) {
    if (areNeighbors(currentHex, hex) && !chain.includes(hex.id)) {
      targets.add(hex.id)
    }
  }
  return targets
})

const chainSet = computed<Set<HexId>>(() => new Set(gameState.turn.chain))

const FIGURE_RADIUS = 14

const figureLayout = computed(() => {
  const map = new Map<HexId, { id: string; color: string; name: string; x: number; y: number; r: number }[]>()
  for (const player of playerList.value) {
    if (!player.position) continue
    if (drag.active && drag.playerId === player.id) continue
    if (!map.has(player.position)) map.set(player.position, [])
    map.get(player.position)!.push({ id: player.id, color: player.color, name: player.name, x: 0, y: 0, r: FIGURE_RADIUS })
  }
  for (const [hexId, group] of map) {
    const hex = gameState.hexes[hexId]
    if (!hex) continue
    const { x: cx, y: cy } = hexToPixel(hex.q, hex.r)
    const n = group.length
    if (n === 1) {
      group[0].x = cx
      group[0].y = cy
    } else if (n === 2) {
      const r = FIGURE_RADIUS * 0.65
      const spacing = FIGURE_RADIUS * 1.2
      const totalWidth = (n - 1) * spacing
      group.forEach((fig, i) => {
        fig.x = cx - totalWidth / 2 + i * spacing
        fig.y = cy
        fig.r = r
      })
    } else {
      const r = FIGURE_RADIUS * 0.65
      const ringR = FIGURE_RADIUS * 0.8
      group.forEach((fig, i) => {
        const angle = (i / n) * Math.PI * 2 - Math.PI / 2
        fig.x = cx + Math.cos(angle) * ringR
        fig.y = cy + Math.sin(angle) * ringR
        fig.r = r
      })
    }
  }
  return map
})

function hexFill(hexId: HexId): string {
  const hex = gameState.hexes[hexId]
  if (!hex) return '#1a1a26'
  // base color shifts with heat
  const heatRatio = hex.heat / MAX_HEAT
  if (heatRatio === 0) return '#1a1a26'
  // interpolate from dark to warm red
  const r = Math.round(26 + heatRatio * (140 - 26))
  const g = Math.round(26 + heatRatio * (40 - 26))
  const b = Math.round(38 + heatRatio * (50 - 38))
  return `rgb(${r},${g},${b})`
}

function heatDots(hexId: HexId): { x: number; y: number }[] {
  const hex = gameState.hexes[hexId]
  if (!hex || hex.heat === 0) return []
  const { x: cx, y: cy } = hexToPixel(hex.q, hex.r)
  const dots: { x: number; y: number }[] = []
  const startY = cy + HEX_SIZE * 0.35
  const spacing = 11
  const totalWidth = (hex.heat - 1) * spacing
  for (let i = 0; i < hex.heat; i++) {
    dots.push({
      x: cx - totalWidth / 2 + i * spacing,
      y: startY,
    })
  }
  return dots
}

const tintClass = computed(() => {
  if (!props.outcomeTint) return ''
  return `tint-${props.outcomeTint}`
})

onMounted(() => {
  window.addEventListener('pointerup', onPointerUp)
  window.addEventListener('pointermove', onPointerMove)
})
onUnmounted(() => {
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointermove', onPointerMove)
})
</script>

<template>
  <div class="board-container" :class="tintClass">
    <svg
      ref="svgRef"
      :viewBox="viewBox"
      class="hex-svg"
      @contextmenu.prevent
    >
      <g v-for="hex in hexList" :key="hex.id">
        <polygon
          :points="hexCorners(hexToPixel(hex.q, hex.r).x, hexToPixel(hex.q, hex.r).y)"
          :fill="hexFill(hex.id)"
          :stroke="selectedHex === hex.id ? 'var(--primary)' : swapTarget === hex.id ? 'var(--accent)' : chainSet.has(hex.id) ? 'var(--primary)' : '#33334a'"
          :stroke-width="selectedHex === hex.id || swapTarget === hex.id || chainSet.has(hex.id) ? 2.5 : 1"
          :class="{
            selected: selectedHex === hex.id,
            'swap-target': swapTarget === hex.id,
            'chain-hex': chainSet.has(hex.id),
            'valid-target': validMoveTargets.has(hex.id),
          }"
          @click="onHexClick($event, hex.id)"
          @contextmenu="onHexContext($event, hex.id)"
        />
        <!-- heat dots -->
        <circle
          v-for="(dot, i) in heatDots(hex.id)"
          :key="i"
          :cx="dot.x"
          :cy="dot.y"
          :r="4"
          fill="#ff6b3a"
          stroke="#cc4a20"
          stroke-width="0.5"
        />
        <!-- heat number -->
        <text
          v-if="gameState.hexes[hex.id]?.heat > 0"
          :x="hexToPixel(hex.q, hex.r).x"
          :y="hexToPixel(hex.q, hex.r).y - HEX_SIZE * 0.2"
          text-anchor="middle"
          class="heat-label"
        >{{ gameState.hexes[hex.id].heat }}</text>
        <!-- inscription -->
        <text
          v-if="gameState.hexes[hex.id]?.inscription"
          :x="hexToPixel(hex.q, hex.r).x"
          :y="hexToPixel(hex.q, hex.r).y + HEX_SIZE * 0.15"
          text-anchor="middle"
          class="hex-inscription"
        >{{ gameState.hexes[hex.id].inscription.length > 12 ? gameState.hexes[hex.id].inscription.slice(0, 11) + '…' : gameState.hexes[hex.id].inscription }}</text>
      </g>

      <!-- figures -->
      <template v-for="[hexId, group] in figureLayout" :key="hexId">
        <g v-for="fig in group" :key="fig.id">
          <circle
            :cx="fig.x"
            :cy="fig.y"
            :r="fig.r"
            :fill="fig.color"
            stroke="#fff"
            stroke-width="2"
            class="figure"
            :class="{ 'my-figure': fig.id === myPlayer?.id, draggable: isMyTurn ? fig.id === myPlayer?.id : gameState.turn.phase !== 'active' }"
            @pointerdown="onFigurePointerDown($event, fig.id)"
          />
          <text
            :x="fig.x"
            :y="fig.y + fig.r * 0.3"
            text-anchor="middle"
            class="figure-label"
            :style="{ fontSize: fig.r * 0.95 + 'px' }"
          >{{ fig.name.charAt(0).toUpperCase() }}</text>
        </g>
      </template>

      <!-- dragged figure -->
      <g v-if="drag.active && drag.playerId">
        <circle
          :cx="drag.x"
          :cy="drag.y"
          :r="14"
          :fill="gameState.players[drag.playerId]?.color || '#888'"
          stroke="#fff"
          stroke-width="2"
          class="figure dragging"
          opacity="0.85"
        />
      </g>
    </svg>

    <!-- hex context panel -->
    <div v-if="selectedHex && gameState.turn.phase !== 'active'" class="hex-panel">
      <div class="hex-panel-title">Feld {{ selectedHex }}</div>
      <div class="hex-panel-row">
        <span>Hitze: {{ gameState.hexes[selectedHex]?.heat ?? 0 }}</span>
        <div class="hex-panel-btns">
          <button class="mini-btn" @click="decrementHeat(selectedHex)" :disabled="(gameState.hexes[selectedHex]?.heat ?? 0) <= 0">−</button>
          <button class="mini-btn" @click="incrementHeat(selectedHex)" :disabled="(gameState.hexes[selectedHex]?.heat ?? 0) >= MAX_HEAT">+</button>
        </div>
      </div>
      <div class="hex-panel-row inscription-row">
        <label class="inscription-label">Inschrift</label>
        <input
          class="inscription-input"
          type="text"
          maxlength="100"
          :value="gameState.hexes[selectedHex]?.inscription ?? ''"
          @input="onInscriptionInput(selectedHex, $event)"
          placeholder="z.B. „Altes Grab""
        />
      </div>
      <button
        v-if="!swapTarget"
        class="swap-btn"
        @click="swapTarget = selectedHex; selectedHex = null"
      >Zum Tauschen wählen</button>
      <button v-else class="swap-btn cancel" @click="swapTarget = null">Tausch abbrechen</button>
      <button class="close-btn" @click="selectedHex = null">Schließen</button>
    </div>

    <!-- swap hint -->
    <div v-if="swapTarget" class="swap-hint">
      Wähle ein zweites Feld zum Tauschen der Hitze mit {{ swapTarget }}
    </div>
  </div>
</template>

<style scoped>
.board-container {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 12px;
  overflow: hidden;
  transition: background-color 0.6s ease;
  background: var(--bg-base);
}
.board-container.tint-success {
  background: linear-gradient(135deg, rgba(212, 166, 74, 0.12), rgba(212, 166, 74, 0.04));
}
.board-container.tint-mixed {
  background: linear-gradient(135deg, rgba(120, 120, 140, 0.12), rgba(120, 120, 140, 0.04));
}
.board-container.tint-failure {
  background: linear-gradient(135deg, rgba(80, 40, 100, 0.18), rgba(80, 40, 100, 0.06));
}
.hex-svg {
  width: 100%;
  height: 100%;
  display: block;
}
polygon {
  transition: fill 0.3s, stroke 0.2s, stroke-width 0.2s;
  cursor: pointer;
}
polygon.valid-target {
  stroke: var(--primary) !important;
  stroke-width: 3 !important;
  filter: drop-shadow(0 0 6px rgba(212, 166, 74, 0.6));
  animation: pulse 1.5s ease-in-out infinite;
}
polygon.chain-hex {
  stroke: var(--accent) !important;
  stroke-width: 2.5 !important;
}
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}
.heat-label {
  fill: #ffaa88;
  font-size: 13px;
  font-weight: 700;
  pointer-events: none;
  user-select: none;
}
.figure {
  cursor: grab;
  transition: filter 0.2s;
}
.figure.draggable {
  cursor: grab;
}
.figure.draggable:hover {
  filter: drop-shadow(0 0 6px rgba(255, 255, 255, 0.5));
}
.figure.dragging {
  cursor: grabbing;
}
.figure-label {
  fill: white;
  font-size: 13px;
  font-weight: 700;
  pointer-events: none;
  user-select: none;
}
.hex-panel {
  position: absolute;
  bottom: 16px;
  left: 16px;
  background: var(--bg-panel);
  border: 1px solid var(--border-light);
  border-radius: 12px;
  padding: 1rem;
  min-width: 200px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  animation: fadeIn 0.2s ease;
  z-index: 10;
}
.hex-panel-title {
  font-size: 0.8rem;
  color: var(--text-dim);
  margin-bottom: 0.5rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.hex-panel-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
  font-size: 0.9rem;
}
.hex-panel-btns {
  display: flex;
  gap: 0.3rem;
}
.mini-btn {
  width: 30px;
  height: 30px;
  padding: 0;
  font-size: 1.1rem;
  display: flex;
  align-items: center;
  justify-content: center;
}
.swap-btn {
  width: 100%;
  margin-bottom: 0.4rem;
  font-size: 0.8rem;
  padding: 0.5em;
}
.inscription-row {
  flex-direction: column;
  align-items: stretch;
  gap: 0.3rem;
  margin-bottom: 0.75rem;
}
.inscription-label {
  font-size: 0.75rem;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.inscription-input {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0.4em 0.6em;
  color: var(--text);
  font-size: 0.85rem;
  width: 100%;
  box-sizing: border-box;
}
.inscription-input:focus {
  outline: none;
  border-color: var(--primary);
}
.hex-inscription {
  fill: var(--text);
  font-size: 10px;
  font-weight: 500;
  pointer-events: none;
  user-select: none;
  opacity: 0.85;
}
.swap-btn.cancel {
  border-color: var(--error);
  color: var(--error);
}
.close-btn {
  width: 100%;
  font-size: 0.8rem;
  padding: 0.5em;
  opacity: 0.7;
}
.swap-hint {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-panel);
  border: 1px solid var(--accent);
  border-radius: 999px;
  padding: 0.5em 1.2em;
  font-size: 0.85rem;
  color: var(--accent);
  animation: fadeIn 0.2s ease;
  z-index: 10;
}
</style>
