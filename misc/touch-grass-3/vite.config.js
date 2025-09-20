import { defineConfig } from 'vite'

export default defineConfig({
  server: {
    proxy: {
      '^/(start|login|update|oauth-check|auth)': 'http://localhost:3000',
    },
    allowedHosts: true,
  },
})
