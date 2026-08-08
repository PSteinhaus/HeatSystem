import type { Hex, HexId } from '../types'

export const HEX_SIZE = 42

export const NEIGHBOR_DIRECTIONS: ReadonlyArray<readonly [number, number]> = [
  [+1, 0],
  [+1, -1],
  [0, -1],
  [-1, 0],
  [-1, +1],
  [0, +1],
]

export function hexId(q: number, r: number): HexId {
  return `${q},${r}`
}

export function parseHexId(id: HexId): { q: number; r: number } {
  const [q, r] = id.split(',').map(Number)
  return { q, r }
}

export function hexToPixel(q: number, r: number, size = HEX_SIZE): { x: number; y: number } {
  const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r)
  const y = size * (3 / 2) * r
  return { x, y }
}

export function hexCorners(cx: number, cy: number, size = HEX_SIZE): string {
  const pts: string[] = []
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 180) * (60 * i - 30)
    pts.push(`${(cx + size * Math.cos(angle)).toFixed(2)},${(cy + size * Math.sin(angle)).toFixed(2)}`)
  }
  return pts.join(' ')
}

export function areNeighbors(a: Hex, b: Hex): boolean {
  return hexDistance(a, b) === 1
}

export function hexDistance(a: Hex, b: Hex): number {
  const dq = a.q - b.q
  const dr = a.r - b.r
  const ds = -dq - dr
  return (Math.abs(dq) + Math.abs(dr) + Math.abs(ds)) / 2
}

export function generateHexes(count: number): Hex[] {
  const all: { q: number; r: number; dist: number }[] = []
  const maxR = Math.ceil(Math.sqrt(count)) + 1
  for (let r = -maxR; r <= maxR; r++) {
    for (let q = -maxR; q <= maxR; q++) {
      const s = -q - r
      const dist = Math.max(Math.abs(q), Math.abs(r), Math.abs(s))
      if (dist <= maxR) {
        all.push({ q, r, dist })
      }
    }
  }
  all.sort((a, b) => a.dist - b.dist || a.q - b.q || a.r - b.r)
  return all.slice(0, count).map(({ q, r }) => ({ id: hexId(q, r), q, r, heat: 0, inscription: '' }))
}

export function computeViewBox(hexes: Hex[], size = HEX_SIZE): string {
  if (hexes.length === 0) return `-100 -100 200 200`
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity
  for (const hex of hexes) {
    const { x, y } = hexToPixel(hex.q, hex.r, size)
    const w = size * Math.sqrt(3)
    const h = size * 2
    minX = Math.min(minX, x - w / 2)
    maxX = Math.max(maxX, x + w / 2)
    minY = Math.min(minY, y - h / 2)
    maxY = Math.max(maxY, y + h / 2)
  }
  const pad = size * 1.2
  return `${(minX - pad).toFixed(1)} ${(minY - pad).toFixed(1)} ${(maxX - minX + pad * 2).toFixed(1)} ${(maxY - minY + pad * 2).toFixed(1)}`
}
