import { app, shell, BrowserWindow, ipcMain, dialog } from 'electron'
import { join } from 'path'
import { desktopChannels } from '../shared/desktop-api'
import { coreChannels } from '@altrex/contracts/channels'

let mainWindow: BrowserWindow | null = null
let splashWindow: BrowserWindow | null = null

function createSplashWindow(): BrowserWindow {
  const splash = new BrowserWindow({
    width: 480,
    height: 320,
    transparent: true,
    frame: false,
    alwaysOnTop: true,
    resizable: false,
    center: true,
    show: false,
    webPreferences: {
      sandbox: true,
      nodeIntegration: false,
      contextIsolation: true,
    },
  })

  const splashHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; user-select: none; }
    body {
      width: 100vw; height: 100vh;
      display: flex; align-items: center; justify-content: center;
      background: transparent;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Inter, sans-serif;
    }
    .splash-card {
      width: 440px; height: 280px;
      background: rgba(18, 20, 24, 0.96);
      backdrop-filter: blur(24px);
      border: 1px solid rgba(255, 255, 255, 0.08);
      border-radius: 28px;
      display: flex; flex-direction: column; align-items: center; justify-content: center;
      box-shadow: 0 30px 90px rgba(0, 0, 0, 0.7), 0 0 1px rgba(255,255,255,0.2) inset;
      color: #fff;
    }
    .logo-box {
      width: 76px; height: 76px;
      border-radius: 22px;
      background: rgba(255, 255, 255, 0.03);
      border: 1px solid rgba(255, 255, 255, 0.08);
      display: flex; align-items: center; justify-content: center;
      margin-bottom: 20px;
      box-shadow: 0 10px 30px rgba(0,0,0,0.3);
    }
    h1 {
      font-size: 20px; font-weight: 700; letter-spacing: 0.12em;
      margin-bottom: 6px;
      background: linear-gradient(135deg, #ffffff 0%, #cbd5e1 100%);
      -webkit-background-clip: text; -webkit-text-fill-color: transparent;
    }
    p { font-size: 11px; color: #64748b; font-weight: 500; letter-spacing: 0.04em; margin-bottom: 24px; }
    .loader-track {
      width: 200px; height: 3px;
      background: rgba(255, 255, 255, 0.06);
      border-radius: 99px; overflow: hidden; position: relative;
    }
    .loader-bar {
      width: 50%; height: 100%;
      background: linear-gradient(90deg, #38bdf8, #10b981);
      border-radius: 99px;
      position: absolute; left: -50%;
      animation: load 1.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
    }
    @keyframes load {
      0% { left: -50%; width: 30%; }
      50% { left: 35%; width: 60%; }
      100% { left: 100%; width: 30%; }
    }
  </style>
</head>
<body>
  <div class="splash-card">
    <div class="logo-box">
      <svg width="44" height="44" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <linearGradient id="g1" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#38bdf8"/>
            <stop offset="50%" stop-color="#3b82f6"/>
            <stop offset="100%" stop-color="#10b981"/>
          </linearGradient>
        </defs>
        <path d="M10 3.5L22 3.5L28.5 10L28.5 22L22 28.5L10 28.5L3.5 22L3.5 10L10 3.5Z" fill="#131517" stroke="url(#g1)" stroke-width="2.2" stroke-linejoin="round"/>
        <path d="M13 11.5L9 16L13 20.5" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M19 11.5L23 16L19 20.5" stroke="#10b981" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <circle cx="16" cy="16" r="2.2" fill="#38bdf8"/>
      </svg>
    </div>
    <h1>OCTREX CODE</h1>
    <p>INITIALIZING AUTONOMOUS CORE…</p>
    <div class="loader-track"><div class="loader-bar"></div></div>
  </div>
</body>
</html>`

  splash.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(splashHtml)}`)
  splash.once('ready-to-show', () => splash.show())
  return splash
}

function createWindow(): void {
  splashWindow = createSplashWindow()

  mainWindow = new BrowserWindow({
    width: 1360,
    height: 860,
    minWidth: 980,
    minHeight: 650,
    show: false,
    autoHideMenuBar: true,
    title: 'OCTREX CODE',
    backgroundColor: '#f4f6f8',
    titleBarStyle: 'hiddenInset',
    webPreferences: {
      preload: join(__dirname, '../preload/index.cjs'),
      sandbox: false,
      contextIsolation: true,
    },
  })

  mainWindow.on('ready-to-show', () => {
    setTimeout(() => {
      if (splashWindow && !splashWindow.isDestroyed()) splashWindow.destroy()
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.show()
        mainWindow.focus()
      }
    }, 450)
  })

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url)
    return { action: 'deny' }
  })

  if (!app.isPackaged && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }
}

// REGISTER IPC HANDLERS
function setupIpc(): void {
  ipcMain.handle(desktopChannels.openProject, async () => {
    if (!mainWindow) return null
    const res = await dialog.showOpenDialog(mainWindow, {
      properties: ['openDirectory'],
      title: 'Open Project in Octrex',
    })
    if (res.canceled || !res.filePaths[0]) return null
    const path = res.filePaths[0]
    const name = path.split(/[\\/]/).filter(Boolean).pop() || 'project'
    return { path, name }
  })

  ipcMain.handle(desktopChannels.attachmentPick, async () => {
    if (!mainWindow) return []
    const res = await dialog.showOpenDialog(mainWindow, {
      properties: ['openFile', 'multiSelections'],
      title: 'Attach Files to Octrex',
    })
    return res.filePaths || []
  })

  ipcMain.handle(desktopChannels.recentProject, async () => null)
  ipcMain.handle(desktopChannels.runtimeInfo, async () => ({ version: '1.4.0', os: process.platform }))
  ipcMain.handle(desktopChannels.providerStatus, async () => [])
  ipcMain.handle(desktopChannels.providerModels, async () => [])
  ipcMain.handle(desktopChannels.providerRefreshModels, async () => [])
  ipcMain.handle(desktopChannels.providerInstallLocalModel, async () => ({ ok: true }))
  ipcMain.handle(desktopChannels.providerDiagnostics, async () => ({}))
  ipcMain.handle(desktopChannels.providerTest, async () => ({ ok: true, latencyMs: 120 }))
  ipcMain.handle(desktopChannels.providerConnect, async () => ({ ok: true }))
  ipcMain.handle(desktopChannels.providerDisconnect, async () => ({ ok: true }))
  ipcMain.handle(desktopChannels.providerOpenExternal, async (_e, _p, _k) => { return true })
  ipcMain.handle(desktopChannels.chatStart, async () => ({ ok: true, taskId: `task-${Date.now()}` }))
  ipcMain.handle(desktopChannels.chatCancel, async () => ({ ok: true }))
  ipcMain.handle(desktopChannels.runRevise, async () => ({ ok: true }))
  ipcMain.handle(desktopChannels.projectRuns, async () => [])

  // Core channels fallback
  ipcMain.handle(coreChannels.command, async (_e, name: string, req: any) => {
    if (name === 'permission.configure') return { ok: true, value: { interactive: true } }
    if (name === 'permission.respond') return { ok: true, value: { approved: req?.decision === 'approve' } }
    if (name === 'consent.list') return { ok: true, value: [] }
    if (name === 'consent.grant') return { ok: true, value: { granted: true } }
    if (name === 'session.list') return { ok: true, value: [] }
    if (name === 'task.start') {
      const taskId = `task-${Date.now()}`
      return { ok: true, value: { taskId, state: 'VERIFIED', changedFiles: [] } }
    }
    if (name === 'task.cancel') return { ok: true, value: { cancelled: true } }
    return { ok: true, value: null }
  })
}

app.whenReady().then(() => {
  setupIpc()
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
