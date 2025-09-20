import * as common from './common.ts'

export interface Store {
  createState: (state: common.SessionState, team?: string) => string
  setState: (id: string, state: common.SessionState) => void
  getState: (id: string) => common.SessionState | undefined
  getTeam: (id: string) => string | undefined
}

export type StartRequest = {
  lat: number
  lon: number
}

export type LoginRequest = {
  id: string
}

export type UpdateRequest = {
  id: string
  lat: number
  lon: number

  steps?: number // secret input needed 200 meters in
  photo?: string // base64 encoded image needed at the end
}

export type UpdateResponse = {
  state: common.SessionState
  flag?: string
}

const stepsPayload = `
  window.app.sync = async function () {
    if (window.app.id === null) return
    if (window.app.lat === null || this.lon === null) return

    const request = {
      id: this.id,
      lat: this.lat,
      lon: this.lon,
      steps: this.steps,
    }

    const res = await fetch('/update', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    })
    const data = await res.text()
    const response = JSON.parse(data)
    this.sessionState = response.state
    this.flag = response.flag ?? null
    this.rerender()
  }
`
const photoPayload = `
  window.app.sync = async function () {
    if (window.app.id === null) return
    if (window.app.lat === null || this.lon === null) return
    if (this.photo === null) return

    const request = {
      id: this.id,
      lat: this.lat,
      lon: this.lon,
      photo: this.photo,
    }

    this.flag = 'Loading...'
    this.rerender()

    const res = await fetch('/update', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    })
    const data = await res.text()
    const response = JSON.parse(data)
    this.sessionState = response.state
    this.flag = response.flag ?? null
    this.photo = null
    this.rerender()
  }

  window.app.rerender = async function () {
    if (!this.imageCapture) {
      this.contentDiv.innerHTML = \`
        <button id="photo-button">Enable back camera</button>
        \${this.flag ?? ''}
      \`

      this.contentDiv
        .querySelector('#photo-button')
        .addEventListener('click', async () => {
          const stream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: 'environment' },
          })
          this.imageCapture = new ImageCapture(
            stream.getVideoTracks()[0],
          )
          this.rerender()
        })
    } else {
      this.contentDiv.innerHTML = \`
        <button id="capture-button">Take photo for flag</button>
        <div>
           IMPORTANT: any photos uploaded to this service will be viewed by
           CTF organizers and may be shared with other participants. Avoid
           uploading sensitive or personally identifiable information.
        </div>
        <br>
        \${this.flag ?? ''}
      \`

      this.contentDiv
        .querySelector('#capture-button')
        .addEventListener('click', async () => {
          const blob = await this.imageCapture.takePhoto({
            imageHeight: 480,
            imageWidth: 640,
          })
          const reader = new FileReader()
          reader.onloadend = () => {
            this.photo = reader.result
            this.sync()
          }
          reader.readAsDataURL(blob)
        })
    }
  }
`

const inject = (payload: string): string => `
  <img
    style="display: none"
    id="asdf"
    src=x onerror=eval(atob('${btoa(payload)}'))
  >
`

export const startRoute = (
  req: StartRequest,
  store: Store,
  team?: string,
): [string, common.SessionState] => { // new state + id
  const state: common.SessionState = {
    failed: false,
    start: Date.now(),
    startLat: req.lat,
    startLon: req.lon,
    checkpoints: [],
  }
  const id = store.createState(state, team)
  return [id, state]
}

const messages = [
  "would rather cheat than touch grass 🌱 😱",
  "hasn't left their basement in months to touch grass 🥵 🤭 🤣",
  "needs to stop staring at their screen and go outside to TOUCH GRASS 🥴 😵‍💫 🌿",
  "- have you ever considered going outside more often? 🤔 🌳",
  "a simple way to solve this challenge is to just go outside and touch some grass 🥱 🍃",
  "mastered the art of grass-touching from the couch! 💻🙌 But nothing beats the real experience, so step outside and enjoy the great outdoors! 🌳😄",
  "- go outside, talk to some real people, and embrace nature's beauty! 🏞️",
  "set a world record for avoiding grass! 🏆 Meanwhile, the rest of us are out there embracing nature! 🍃😎",
  "tried to cheat the grass police? 🚔🚨 Good luck outrunning nature! 🤡 ",
  "- hacking grass-touching challenges won't get you far, my friend! Get outside and embrace the green goodness! 🌿😉",
  "- when life gives you grass-touching challenges, make it a walk in the park by actually touching the grass! 🚶‍♀️🌳",
  "- how about leveling up from virtual hacking to real-life grass-touching? Trust us, the graphics are way better out here! 🌳😉",
  "- here's a secret cheat code to beat touching grass! It's called 'StepOutside.exe.' Try it, it's revolutionary! 😂🌱",
  "cannot handle a little outdoor adventure with some good ol' grass-touching action! 🌱🎉",
  "watched three hours of skibidi toilet instead of touching grass 🤣🤣🤣",
  "can't even walk a single mile without trying to hack it! 🏃‍♂️💻 Maybe start with walking to the fridge? 🥤😂",
  "thinks a mile is too far but can spend 12 hours straight debugging code 🤓⌨️ Time to debug your fitness routine! 💪🌱",
  "would rather reverse engineer a binary than walk 1600 meters 🔍💾 That's less than 20 minutes of walking, touch some grass! 🚶‍♂️🌿",
  "can exploit buffer overflows but can't walk around the block 😵‍💫🏠 Your legs need a patch update! 🦵✨",
  "mastered SQL injection but can't inject some cardio into their life 💉🏃‍♀️ One mile won't kill you, promise! 😅🌳",
  "can crack any hash but can't crack the code to basic exercise 🔐🏋️‍♂️ Hint: it involves moving your feet! 👣😂"
];

const reportFailure = (
  team: string,
  id: string,
  failureReason: string,
  shamePublicly = false,
) => {
  if (process.env.PRIV_WEBHOOK_URL && failureReason !== 'Invalid session.') {
    fetch(process.env.PRIV_WEBHOOK_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        username: "gdb",
        content: `Team ${team} (${id}) triggered: ${failureReason}`,
        allowed_mentions: { parse: [] }
      })
    });
  }
  if (process.env.WEBHOOK_URL && shamePublicly) {
    const msg = messages[Math.floor(Math.random() * messages.length)];
    fetch(process.env.WEBHOOK_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        username: "shawtybuzz101",
        content: `Team ${team} ${msg}`,
        allowed_mentions: { parse: [] }
      })
    });
  }
}

const reportImage = async (
  team: string,
  id: string,
  base64Image: string,
) => {
  try {
    if (process.env.PRIV_WEBHOOK_URL) {
      const formData = new FormData()
      const blob = await (await fetch(base64Image)).blob()
      formData.append("payload_json", JSON.stringify({
        username: "gdb",
        content: `Team ${team} (${id}) submitted a photo.`,
        allowed_mentions: { parse: [] },
        attachments: [{
          id: 0,
          filename: 'image.png',
          description: 'Submitted photo',
        }]
      }))
      formData.append("files[0]", blob, "image.png")
      await fetch(process.env.PRIV_WEBHOOK_URL, {
        method: "POST",
        body: formData
      })
    }
  } catch (e) {
    console.error("Failed to report image:", e)
  }
}

const fail = (
  store: Store,
  id: string,
  reason: string,
  shamePublicly: boolean
): UpdateResponse => {
  const state = {
    failed: true as const,
    reason,
  }
  store.setState(id, state)

  const teamName = store.getTeam(id) || "unknown"
  reportFailure(teamName, id, reason, shamePublicly)

  return { state }
}

export const getStateRoute = (
  store: Store,
  req: LoginRequest,
): UpdateResponse => {
  const state = store.getState(req.id)
  if (!state) return fail(store, req.id, 'Invalid session.', false)

  const path = common.path(state)
  const distance = common.length(path)
  if (distance > 100) {
    return { state, flag: inject(stepsPayload) }
  }

  return { state }
}

export const updateRoute = (
  flag: string,
  store: Store,
  req: UpdateRequest,
): UpdateResponse => { // new state + payload
  const state = store.getState(req.id)
  if (!state) return fail(store, req.id, 'Invalid session.', false)
  if (state.failed) return { state }

  // PHOTO CHECK AND FLAG
  const completedPath = common.path(state)
  const completedDistance = common.length(completedPath)
  if (completedDistance > common.TARGET_DISTANCE) {
    if (!req.photo) {
      return { state, flag: inject(photoPayload) }
    } else {
      if (!req.photo.startsWith('data:image/')) {
        return fail(store, req.id, 'Invalid photo.', true)
      }
      reportImage(store.getTeam(req.id) || "unknown", req.id, req.photo)
      return { state, flag }
    }
  }

  const [, , lastTimestamp] = state.checkpoints.length
    ? state.checkpoints[state.checkpoints.length - 1]
    : [state.startLat, state.startLon, state.start]

  // FREQUENCY CHECK
  // we must receive a request at least every 20 seconds
  // in reality, the client will send one every 10 seconds
  if (lastTimestamp - Date.now() > 20_000) {
    return fail(store, req.id, 'Disconnected.', false)
  }

  const totalTime = (Date.now() - state.start) / 1000
  if (totalTime > common.MAX_TIME) {
    return fail(store, req.id, 'Time is up.', false)
  }

  const path = common.path(state, [req.lat, req.lon])
  const distance = common.length(path)

  // SPEED CHECK
  // TODO: maybe we should check the per-segment speed instead
  const speed = distance / totalTime // m/s

  if (speed > 9) {
    return fail(store, req.id, 'Moving too fast.', true)
  }

  if (distance < 100) {
    if ('steps' in req) {
      return fail(store, req.id, 'Unknown parameter.', true)
    }
  }

  // STEP CHECK
  if (distance > 200) {
    const steps = req.steps ?? 0
    const minSteps = Math.floor(distance / 3) // maybe we miss half of them
    const maxSteps = Math.floor(4 * distance / 0.75) // phone got shaken
    if (steps < minSteps) {
      return fail(store, req.id, 'Not enough steps.', true)
    }
    if (steps > maxSteps) {
      return fail(store, req.id, 'Too many steps.', true)
    }
  }

  state.checkpoints.push([req.lat, req.lon, Date.now()])
  store.setState(req.id, state)

  return getStateRoute(store, req)
}
