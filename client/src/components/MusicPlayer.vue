<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { gameState, setMusicTrack } from '../store'
import type { MusicTrackId } from '../types'

const TRACKS: { id: MusicTrackId; label: string }[] = [
  { id: 'none', label: 'Keine Musik' },
  { id: 'ambient', label: 'Ambient' },
  { id: 'forest', label: 'Wald' },
  { id: 'mystic', label: 'Mystisch' },
]

const volume = ref(0.5)
const expanded = ref(false)

let audioCtx: AudioContext | null = null
let masterGain: GainNode | null = null
let oscillators: OscillatorNode[] = []
let lfo: OscillatorNode | null = null
let lfoGain: GainNode | null = null
let noiseSource: AudioBufferSourceNode | null = null
let noiseGain: GainNode | null = null
let currentTrack: MusicTrackId = 'none'

function ensureContext() {
  if (!audioCtx) {
    audioCtx = new AudioContext()
    masterGain = audioCtx.createGain()
    masterGain.gain.value = volume.value
    masterGain.connect(audioCtx.destination)
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume()
  }
}

function stopAll() {
  for (const osc of oscillators) {
    try { osc.stop() } catch { /* already stopped */ }
    osc.disconnect()
  }
  oscillators = []
  if (lfo) {
    try { lfo.stop() } catch { /* already stopped */ }
    lfo.disconnect()
    lfo = null
  }
  if (lfoGain) {
    lfoGain.disconnect()
    lfoGain = null
  }
  if (noiseSource) {
    try { noiseSource.stop() } catch { /* already stopped */ }
    noiseSource.disconnect()
    noiseSource = null
  }
  if (noiseGain) {
    noiseGain.disconnect()
    noiseGain = null
  }
}

function createNoiseBuffer(ctx: AudioContext): AudioBuffer {
  const bufferSize = ctx.sampleRate * 2
  const buffer = ctx.createBuffer(1, bufferSize, ctx.sampleRate)
  const data = buffer.getChannelData(0)
  for (let i = 0; i < bufferSize; i++) {
    data[i] = Math.random() * 2 - 1
  }
  return buffer
}

function playTrack(track: MusicTrackId) {
  ensureContext()
  stopAll()
  if (!audioCtx || !masterGain) return
  currentTrack = track

  if (track === 'none') return

  if (track === 'ambient') {
    const freqs = [110, 164.81, 220]
    for (const freq of freqs) {
      const osc = audioCtx.createOscillator()
      osc.type = 'sine'
      osc.frequency.value = freq
      const gain = audioCtx.createGain()
      gain.gain.value = 0.15
      osc.connect(gain)
      gain.connect(masterGain)
      osc.start()
      oscillators.push(osc)
    }
  } else if (track === 'forest') {
    // low drone + filtered noise (wind-like)
    const osc = audioCtx.createOscillator()
    osc.type = 'sine'
    osc.frequency.value = 55
    const gain = audioCtx.createGain()
    gain.gain.value = 0.2
    osc.connect(gain)
    gain.connect(masterGain)
    osc.start()
    oscillators.push(osc)

    noiseSource = audioCtx.createBufferSource()
    noiseSource.buffer = createNoiseBuffer(audioCtx)
    noiseSource.loop = true
    const filter = audioCtx.createBiquadFilter()
    filter.type = 'lowpass'
    filter.frequency.value = 400
    noiseGain = audioCtx.createGain()
    noiseGain.gain.value = 0.08
    noiseSource.connect(filter)
    filter.connect(noiseGain)
    noiseGain.connect(masterGain)
    noiseSource.start()
  } else if (track === 'mystic') {
    // detuned oscillators with slow LFO for shimmer
    const freqs = [146.83, 220, 293.66]
    for (const freq of freqs) {
      const osc = audioCtx.createOscillator()
      osc.type = 'triangle'
      osc.frequency.value = freq
      const gain = audioCtx.createGain()
      gain.gain.value = 0.1
      osc.connect(gain)
      gain.connect(masterGain)
      osc.start()
      oscillators.push(osc)
    }
    // LFO modulating master amplitude
    lfo = audioCtx.createOscillator()
    lfo.frequency.value = 0.15
    lfoGain = audioCtx.createGain()
    lfoGain.gain.value = 0.05
    lfo.connect(lfoGain)
    lfoGain.connect(masterGain.gain)
    lfo.start()
  }
}

function onTrackChange(event: Event) {
  const target = event.target as HTMLSelectElement
  const track = target.value as MusicTrackId
  setMusicTrack(track)
}

function onVolumeChange(event: Event) {
  const target = event.target as HTMLInputElement
  volume.value = parseFloat(target.value)
  if (masterGain && audioCtx) {
    masterGain.gain.setTargetAtTime(volume.value, audioCtx.currentTime, 0.05)
  }
}

// React to shared track changes from any player
watch(() => gameState.musicTrack, (track) => {
  if (track !== currentTrack) {
    playTrack(track)
  }
})

onUnmounted(() => {
  stopAll()
  if (audioCtx) {
    audioCtx.close()
    audioCtx = null
  }
})
</script>

<template>
  <div class="music-player" :class="{ expanded }">
    <button class="music-toggle" @click="expanded = !expanded">
      <span class="music-icon">♪</span>
      <span class="music-label">Musik</span>
    </button>
    <div v-if="expanded" class="music-controls">
      <select class="track-select" :value="gameState.musicTrack" @change="onTrackChange">
        <option v-for="track in TRACKS" :key="track.id" :value="track.id">{{ track.label }}</option>
      </select>
      <div class="volume-row">
        <span class="volume-icon">🔊</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          :value="volume"
          class="volume-slider"
          @input="onVolumeChange"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.music-player {
  position: absolute;
  top: 1rem;
  right: 1rem;
  z-index: 45;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.5rem;
}
.music-toggle {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 0.4em 0.9em;
  font-size: 0.85rem;
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s;
}
.music-toggle:hover {
  border-color: var(--primary);
  box-shadow: 0 0 12px rgba(212, 166, 74, 0.2);
}
.music-icon {
  font-size: 1rem;
  color: var(--primary);
}
.music-controls {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  background: var(--bg-panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 0.75rem 1rem;
  min-width: 200px;
  animation: fadeIn 0.2s ease;
}
.track-select {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0.4em 0.6em;
  color: var(--text);
  font-size: 0.85rem;
  cursor: pointer;
}
.track-select:focus {
  outline: none;
  border-color: var(--primary);
}
.volume-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.volume-icon {
  font-size: 0.9rem;
}
.volume-slider {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: var(--border);
  border-radius: 999px;
  outline: none;
  cursor: pointer;
}
.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--primary);
  cursor: pointer;
}
.volume-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--primary);
  border: none;
  cursor: pointer;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
