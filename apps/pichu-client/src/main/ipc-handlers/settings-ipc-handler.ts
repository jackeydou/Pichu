import { ipcMain, shell } from 'electron'
import type { OpenAIOAuthStatus } from '../../shared/openai-oauth.js'
import { hasOpenAIOAuthCredential, loginOpenAIOAuth, logoutOpenAIOAuth } from '../openai-oauth.js'
import {
  clearImageGenerationApiKey,
  getImageGenerationConfigStatus,
  saveImageGenerationApiKey
} from '../stores/image-generation-config-store.js'
import {
  deleteUserModelConfig,
  listOpenAIOAuthModels,
  listUserModelSummaries,
  saveUserModelConfig,
  setOpenAIOAuthEnabledModels
} from '../stores/model-config-store.js'
import {
  applySettingsPatch,
  getSettingsForRenderer,
  type SettingsPatch
} from '../stores/settings-store.js'
import { setCursorOriginHint } from '../tools/computer-use/cursor-overlay.js'

export function registerSettingsIpcHandlers(): void {
  const openAIOAuthStatus = (): OpenAIOAuthStatus => ({
    signedIn: hasOpenAIOAuthCredential(),
    models: listOpenAIOAuthModels()
  })

  ipcMain.handle('settings:get', () => getSettingsForRenderer())

  // Cursor-overlay origin hint: the renderer reports the chat input's
  // on-screen position so the ghost cursor first materialises right at
  // the input, then animates outward to the click target. Receives `null`
  // when no input is mounted (the overlay falls back to the main-window
  // bottom-centre).
  ipcMain.handle('cursor-overlay:set-origin', (_, point: { x: number; y: number } | null) => {
    setCursorOriginHint(point)
  })

  ipcMain.handle('settings:set', (_, patch: SettingsPatch) => applySettingsPatch(patch))
  ipcMain.handle('models:list', () => listUserModelSummaries())
  ipcMain.handle('models:save', (_, input: { model?: unknown; previousId?: unknown }) =>
    saveUserModelConfig(input?.model, input?.previousId)
  )
  ipcMain.handle('models:delete', (_, modelId: unknown) => deleteUserModelConfig(modelId))
  ipcMain.handle('openai-oauth:get', openAIOAuthStatus)
  ipcMain.handle('openai-oauth:login', async () => {
    await loginOpenAIOAuth((url) => shell.openExternal(url))
    return openAIOAuthStatus()
  })
  ipcMain.handle('openai-oauth:logout', async () => {
    setOpenAIOAuthEnabledModels([])
    await logoutOpenAIOAuth()
    return openAIOAuthStatus()
  })
  ipcMain.handle('openai-oauth:set-enabled-models', (_, modelIds: unknown) => {
    setOpenAIOAuthEnabledModels(modelIds)
    return openAIOAuthStatus()
  })
  ipcMain.handle('image-generation-config:get', () => getImageGenerationConfigStatus())
  ipcMain.handle('image-generation-config:save', (_, apiKey: unknown) =>
    saveImageGenerationApiKey(apiKey)
  )
  ipcMain.handle('image-generation-config:clear', () => clearImageGenerationApiKey())
}
