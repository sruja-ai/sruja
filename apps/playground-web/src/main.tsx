import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

// Handle Vite preload errors (e.g., dynamic import failures)
window.addEventListener('vite:preloadError', (event) => {
  console.warn('Vite preload error detected, reloading...', event)
  window.location.reload()
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)

