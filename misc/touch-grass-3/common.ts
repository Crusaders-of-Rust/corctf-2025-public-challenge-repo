export const MAX_TIME = 10 * 60
export const TARGET_DISTANCE = 1600

export type SessionState = {
  failed: false
  start: number // timestamp
  startLat: number
  startLon: number

  checkpoints: [number, number, number][] // [lat, lon, timestamp][]
} | {
  failed: true
  reason: string
}

export const path = (
  state: SessionState,
  next?: [number, number]
): [number, number][] => {
  if (state.failed) return []
  return [
    [state.startLat, state.startLon],
    ...((): [number, number][] => (
      state.checkpoints.map(([lat, lon]) => [lat, lon])
    ))(),
    ...(next ? [next] : []),
  ]
}

const haversine = (lat1: number, lon1: number, lat2: number, lon2: number) => {
  const R = 6371e3
  const phi1 = lat1 * Math.PI / 180
  const phi2 = lat2 * Math.PI / 180
  const deltaPhi = (lat2 - lat1) * Math.PI / 180
  const deltaLambda = (lon2 - lon1) * Math.PI / 180

  const a = (
    Math.sin(deltaPhi / 2) * Math.sin(deltaPhi / 2)
    + Math.cos(phi1)
    * Math.cos(phi2)
    * Math.sin(deltaLambda / 2)
    * Math.sin(deltaLambda / 2)
  )

  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))

  return R * c
};

export const length = (path: [number, number][]) => {
  const windows = path.slice(1).map((coord, i) => [path[i], coord])
  return windows.reduce((a, [from, to]) => a + haversine(...from, ...to), 0)
}
