import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  build: {
    // Configure esbuild to suppress CSS syntax warnings
    // These are false positives from the minifier about modern CSS properties like 'gap'
    cssMinify: 'esbuild',
  },
  esbuild: {
    logOverride: {
      'css-syntax-error': 'silent',
    },
  },
})
