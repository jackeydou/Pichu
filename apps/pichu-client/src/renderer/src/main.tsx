import './assets/main.css'
import './lib/i18n'

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import { applyThemeMode } from './lib/theme'

document.documentElement.dataset.platform = window.electron?.process?.platform ?? 'darwin'
applyThemeMode()

const rootEl = document.getElementById('root')
if (!rootEl) {
  throw new Error('Root element #root not found')
}
createRoot(rootEl).render(
  <StrictMode>
    <App />
  </StrictMode>
)
