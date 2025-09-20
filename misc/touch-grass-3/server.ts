import express from 'express'
import sqlite3 from 'better-sqlite3'
import { z } from 'zod'
import session from 'express-session'
import crypto from 'crypto'

import { getStateRoute, startRoute, updateRoute, type Store } from './server-logic.ts'
import { type SessionState } from './common.ts'

declare module 'express-session' {
  interface SessionData {
    state?: string
    team?: string
    error?: string
  }
}

const stateSchema = z.union([
  z.object({
    failed: z.literal(false),
    start: z.number(),
    startLat: z.number().min(-90).max(90),
    startLon: z.number().min(-180).max(180),
    checkpoints: z.array(z.tuple([
      z.number().min(-90).max(90),
      z.number().min(-180).max(180),
      z.number(),
    ])),
  }),
  z.object({
    failed: z.literal(true),
    reason: z.string(),
  }),
])

const startSchema = z.object({
  lat: z.number().min(-90).max(90),
  lon: z.number().min(-180).max(180),
})

const getStateSchema = z.object({
  id: z.string(),
})

const updateSchema = z.object({
  id: z.string(),
  lat: z.number().min(-90).max(90),
  lon: z.number().min(-180).max(180),
  steps: z.number().min(0).optional(),
  photo: z.string().optional(),
})

class SqliteStore implements Store {
    db: sqlite3.Database
    constructor(db: string) {
        this.db = sqlite3(db)
        this.db.prepare(`
          CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            state TEXT NOT NULL,
            team TEXT
          )
        `).run()
    }

    createState(state: SessionState, team?: string): string {
      const id = crypto.randomUUID()
      this.db.prepare(`
        INSERT INTO sessions (id, state, team) VALUES (?, ?, ?)
      `).run(id, JSON.stringify(state), team || null)
      return id
    }

    setState(id: string, state: SessionState): void {
      this.db.prepare(`
        UPDATE sessions SET state = ? WHERE id = ?
      `).run(JSON.stringify(state), id)
    }

    getState(id: string): SessionState | undefined {
      const row = this.db.prepare(`
        SELECT state FROM sessions WHERE id = ?
      `).get(id) as { state: string }
      if (!row) return undefined
      return stateSchema.parse(JSON.parse(row.state))
    }

    getTeam(id: string): string | undefined {
      const row = this.db.prepare(`
        SELECT team FROM sessions WHERE id = ?
      `).get(id) as { team: string | null }
      return row?.team || undefined
    }
}

const store = new SqliteStore(':memory:')
const app = express()
app.use(express.json({ limit: '5mb' }))

app.use(session({
  secret: process.env.SESSION_SECRET || crypto.randomBytes(64).toString('hex'),
  resave: false,
  saveUninitialized: false,
  cookie: { secure: false, maxAge: 1000 * 60 * 60 } // 1 hour
}))

app.use('/', express.static('dist'))

app.get('/oauth-check', (req, res) => {
  if (!req.session.team) {
    return res.status(401).json({ error: 'OAuth authentication required' })
  }
  res.json({ authenticated: true, team: req.session.team })
})

app.get('/oauth', (req, res) => {
  const state = crypto.randomUUID()
  req.session.state = state
  const redirectUri = `https://${req.get('host')}/auth`
  const oauthUrl = `https://2025.cor.team/auth.html?state=${state}&redirect_uri=${encodeURIComponent(redirectUri)}`
  res.redirect(oauthUrl)
})

app.get('/auth', async (req, res) => {
  if (!req.query.state || typeof req.query.state !== 'string' || req.query.state !== req.session.state) {
    return res.send('missing oauth state')
  }
  if (!req.query.token || typeof req.query.token !== 'string') {
    return res.send('missing oauth token')
  }

  try {
    const r = await fetch('https://2025.cor.team/api/v1/users/me', {
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${req.query.token}`
      }
    })

    const json = await r.json()

    if (json.kind !== 'goodUserData') {
      req.session.error = 'invalid oauth token'
      return res.redirect('/')
    }

    req.session.team = json.data.name
    res.redirect('/')
  } catch (error) {
    req.session.error = 'oauth verification failed'
    res.redirect('/')
  }
})

app.post('/start', (req, res) => {
  if (!req.session.team) {
    return res.status(401).json({ error: 'authentication required' })
  }

  const result = startSchema.safeParse(req.body)
  if (!result.success) return res.status(400).json({ error: 'Invalid request' })
  const startReq = result.data

  const [id, state] = startRoute(startReq, store, req.session.team)
  res.json({ id, state })
})

app.post('/login', (req, res) => {
  const result = getStateSchema.safeParse(req.body)
  if (!result.success) {
    return res.status(400).json({ error: 'Invalid request' })
  }
  const getStateReq = result.data
  const response = getStateRoute(
    store,
    getStateReq,
  )

  return res.end(JSON.stringify(response))
})

app.post('/update', (req, res) => {
  const result = updateSchema.safeParse(req.body)
  if (!result.success) {
    return res.status(400).json({ error: 'Invalid request' })
  }
  const updateReq = result.data
  const response = updateRoute(
    process.env.FLAG ?? 'corctf{test}',
    store,
    updateReq,
  )

  return res.end(JSON.stringify(response))
})

app.listen(3000)
