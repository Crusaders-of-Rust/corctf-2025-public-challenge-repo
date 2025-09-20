import * as L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import './index.css'

import { App } from './client-logic'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div class="content"></div>
  <div class="container">
    <div id="map"></div>
  </div>
`
const map = L.map('map', {
  center: [36.1505276,-115.2439806],
  zoom: 19,
})

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: `
    &copy;
    <a href="http://www.openstreetmap.org/copyright">
      OpenStreetMap
    </a>
  `,
}).addTo(map)

declare global {
  interface Window {
    app: App
    Accelerometer: any
  }
}

const save = localStorage.getItem('save')
window.app = new App(
  save && JSON.parse(save),
  map,
  document.querySelector('.content')!,
  (id) => {
    if (id === null) {
      localStorage.removeItem('save')
    } else {
      localStorage.setItem('save', JSON.stringify(id))
    }
  },
)

let count = 0
setInterval(() => {
  if (count % 5 === 0) {
    window.app.login()
    window.app.sync()
  } else window.app.rerender()
  count += 1
}, 1000)

window.app.rerender()

let down = false
const onAccelerometer = (x: number, y: number, z: number) => {
  const magnitude = x ** 2 + y ** 2 + z ** 2
  if (magnitude < 6 ** 2) {
    if (down == false) {
      window.app.addSteps(1)
      down = true
    }
  } else if (magnitude > 12 ** 2) {
    if (down == true) {
      down = false
    }
  }
}

if ('geolocation' in navigator && 'Accelerometer' in window) {
  // Location
  navigator.geolocation.watchPosition((pos) => {
    window.app.setPosition(pos.coords.latitude, pos.coords.longitude)
  }, (err) => {
    console.log(err)
  }, {
    enableHighAccuracy: true,
  })
  navigator.geolocation.getCurrentPosition((pos) => {
    window.app.setPosition(pos.coords.latitude, pos.coords.longitude)
  }, (err) => {
    console.log(err)
  }, {
    enableHighAccuracy: true,
  })

  // Accelerometer
  const acc = new window.Accelerometer({
    frequency: 60,
    referenceFrame: 'device',
  })

  // Permission feedback
  void (async () => {
    const geolocation = await navigator.permissions.query({
      name: 'geolocation'
    })
    const accelerometer = await navigator.permissions.query({
      name: 'accelerometer' as 'geolocation' // erm
    })
    let hasAccelerometer = false

    const handler = () => {
      const ok = (
        geolocation.state === 'granted'
        && accelerometer.state === 'granted'
      )
      if (!hasAccelerometer) window.app.setPermission('no-accelerometer')
      else if (ok) window.app.setPermission('good')
      else window.app.setPermission('bad')
    }

    acc.addEventListener('reading', () => {
      hasAccelerometer = true
      onAccelerometer(acc.x, acc.y, acc.z)
      handler()
    })
    acc.start()

    geolocation.addEventListener('change', handler)
    accelerometer.addEventListener('change', handler)
    handler()
  })()
}
