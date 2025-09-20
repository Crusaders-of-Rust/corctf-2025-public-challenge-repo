import * as L from 'leaflet'
import * as common from './common'
import {
  type LoginRequest,
  type UpdateRequest,
  type UpdateResponse,
} from './server-logic'

export type PermissionStatus =
  | 'no-accelerometer'
  | 'bad'
  | 'good'

export type SaveState = {
  id: string
  steps: number
}

export class App {
  // app state
  sessionState: common.SessionState | null = null
  flag: string | null = null
  id: string | null = null
  lat: number | null = null
  lon: number | null = null
  steps: number = 0 // TODO: use this
  permission: PermissionStatus = 'bad'

  // effects
  map: L.Map
  contentDiv: HTMLDivElement
  layers: L.Layer[] = []
  saveStateChange: (data: SaveState | null) => void

  constructor(
    save: SaveState | null,
    map: L.Map,
    contentDiv: HTMLDivElement,
    saveStateChange: (data: SaveState | null) => void,
  ) {
    if (save !== null) {
      this.id = save.id
      this.steps = save.steps
    }
    this.map = map
    this.contentDiv = contentDiv
    this.saveStateChange = saveStateChange
  }

  rerender(resize?: boolean) {
    if (this.permission !== 'good') {
      if (this.permission = 'no-accelerometer') {
        this.contentDiv.innerHTML = `
          Accelerometer not found. Are you on Chrome Android?
        `
      } else {
        this.contentDiv.innerHTML = `
          Permissions required.
        `
      }
    } else if (this.lat === null || this.lon === null) {
      this.contentDiv.innerHTML = `
        Awaiting location data...
      `
    } else if (
      this.id === null
      || this.sessionState === null
      || this.sessionState.failed
    ) {
      // null id => never logged in
      // id, null session state => logging in
      // id, session state => logged in

      if (this.id === null) {
        this.contentDiv.innerHTML = `
          <button class="start-button">
            Start run
          </button>
        `
      } else if (this.sessionState === null) {
        this.contentDiv.innerHTML = `
          Logging in...
        `
      } else if (this.sessionState.failed) {
        this.contentDiv.innerHTML = `
          <div class="failed">
            <div>
            ${this.sessionState.reason}
            </div>
            <button class="restart-button">
              Restart run
            </button>
          </div>
        `
      }
    } else {
      const path = common.path(this.sessionState, [this.lat, this.lon])

      const remainingTime = Math.max(
        0,
        common.MAX_TIME * 1000 - (Date.now() - this.sessionState.start),
      )
      const mins = Math.floor(remainingTime / 60000)
      const secs = Math.floor((remainingTime % 60000) / 1000)

      const distance = common.length(path)
      const miles = (distance * 0.0006213712).toPrecision(3)

      this.contentDiv.innerHTML = `
        <div class="status">
          <div>
            Remaining time ${mins}:${secs.toString().padStart(2, '0')}
          </div>
          <div>
            Total distance ${miles} miles
          </div>
          <div>
            Travelled ${this.steps} steps
          </div>
          <button class="restart-button">
            Restart run
          </button>
        </div>
      `

      const polyline = this.drawMap(L.polyline(path, {
        color: 'red',
        weight: 12,
      }))
      if (resize) this.map.fitBounds(polyline.getBounds())
    }

    if (this.flag !== null) {
      this.contentDiv.innerHTML += this.flag
    }

    this.contentDiv
      .querySelector<HTMLButtonElement>('.start-button')
      ?.addEventListener('click', async () => {
        await this.start()
      })

    this.contentDiv
      .querySelector<HTMLButtonElement>('.restart-button')
      ?.addEventListener('click', async () => {
        this.saveStateChange(null)
        await new Promise((r) => setTimeout(r, 100))
        window.location.reload()
      })
  }

  drawMap<T extends L.Layer>(layer: T): T {
    this.layers.forEach((l) => l.remove())
    this.layers = [layer]
    return layer.addTo(this.map)
  }

  addSteps(n: number) {
    if (this.sessionState && !this.sessionState.failed) {
      this.steps += n
      this.broadcastSave()
      this.rerender()
    }
  }

  setPosition(lat: number, lon: number) {
    this.lat = lat
    this.lon = lon
    this.rerender()
  }

  setPermission(permission: PermissionStatus) {
    this.permission = permission
    this.rerender()
  }

  broadcastSave() {
    if (this.id === null) return
    this.saveStateChange({
      id: this.id,
      steps: this.steps,
    })
  }

  async start() {
    if (this.lat === null || this.lon === null) return
    
    // Check if OAuth is required first
    const authCheck = await fetch('/oauth-check')
    if (authCheck.status === 401) {
      window.location.href = '/oauth'
      return
    }
    
    const request = {
      lat: this.lat,
      lon: this.lon,
    }
    const res = await fetch('/start', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    })
    
    if (res.status === 401) {
      window.location.href = '/oauth'
      return
    }
    
    const data = await res.text()
    const response: {
      id: string
      state: common.SessionState
    } = eval(`(${data})`)
    this.id = response.id
    this.broadcastSave()
    this.sessionState = response.state
    this.steps = 0
    this.rerender(true)
  }

  async login() {
    if (this.id === null) return
    if (this.sessionState !== null) return
    if (this.lat === null || this.lon === null) return

    const request: LoginRequest = {
      id: this.id,
    }

    const res = await fetch('/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    })
    const data = await res.text()
    const response: UpdateResponse = JSON.parse(data)
    this.flag = response.flag ?? null
    this.rerender()
    await new Promise((r) => setTimeout(r, 1000))

    this.sessionState = response.state
    this.rerender(true)
    this.sync()
  }

  async sync() {
    if (this.id === null) return
    if (this.sessionState === null) return
    if (this.lat === null || this.lon === null) return

    const request: UpdateRequest = {
      id: this.id,
      lat: this.lat,
      lon: this.lon,
    }

    const res = await fetch('/update', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    })
    const data = await res.text()
    const response: UpdateResponse = JSON.parse(data)
    this.sessionState = response.state
    this.flag = response.flag ?? null
    this.rerender()
  }
}
