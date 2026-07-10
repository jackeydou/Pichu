import { safeStorage } from 'electron'
import {
  isModelApiSpec,
  type UserModelConfig,
  type UserModelSummary
} from '../../shared/model-config.js'
import { deleteStoredSetting, getStoredSetting, setStoredSetting } from './settings-store.js'

const MODELS_SETTING_KEY = 'userModels'
const DEFAULT_CONTEXT_WINDOW = 128_000
const DEFAULT_MAX_TOKENS = 16_384

function requiredString(value: unknown, label: string): string {
  if (typeof value !== 'string' || !value.trim()) throw new Error(`${label} is required`)
  return value.trim()
}

function positiveInteger(value: unknown, fallback: number, label: string): number {
  const resolved = value === undefined ? fallback : value
  if (typeof resolved !== 'number' || !Number.isInteger(resolved) || resolved <= 0) {
    throw new Error(`${label} must be a positive integer`)
  }
  return resolved
}

function normalizeModel(value: unknown): UserModelConfig {
  if (!value || typeof value !== 'object') throw new Error('Model configuration is invalid')
  const input = value as Record<string, unknown>
  const id = requiredString(input.id, 'Model ID')
  const name = requiredString(input.name, 'Display name')
  if (!isModelApiSpec(input.api)) throw new Error('API specification is not supported')
  const baseUrl = requiredString(input.baseUrl, 'Base URL').replace(/\/+$/, '')
  const url = new URL(baseUrl)
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error('Base URL must use http or https')
  }
  return {
    id,
    name,
    api: input.api,
    baseUrl,
    apiKey: typeof input.apiKey === 'string' ? input.apiKey.trim() : '',
    contextWindow: positiveInteger(input.contextWindow, DEFAULT_CONTEXT_WINDOW, 'Context window'),
    maxTokens: positiveInteger(input.maxTokens, DEFAULT_MAX_TOKENS, 'Maximum output tokens'),
    reasoning: input.reasoning === true,
    supportsImages: input.supportsImages === true
  }
}

function decryptApiKey(value: unknown): string {
  if (typeof value !== 'string' || !value) return ''
  try {
    return safeStorage.decryptString(Buffer.from(value, 'base64'))
  } catch {
    return ''
  }
}

function serializeModels(models: UserModelConfig[]): string {
  return JSON.stringify(
    models.map(({ apiKey, ...model }) => ({
      ...model,
      apiKeyEncrypted: apiKey ? safeStorage.encryptString(apiKey).toString('base64') : ''
    }))
  )
}

export function getUserModelConfigs(): UserModelConfig[] {
  const stored = getStoredSetting(MODELS_SETTING_KEY)
  if (!stored) return []
  try {
    const values: unknown = JSON.parse(stored)
    if (!Array.isArray(values)) return []
    const models: UserModelConfig[] = []
    for (const value of values) {
      try {
        const stored = value as Record<string, unknown>
        const model = normalizeModel({ ...stored, apiKey: decryptApiKey(stored.apiKeyEncrypted) })
        if (!models.some((candidate) => candidate.id === model.id)) models.push(model)
      } catch {
        // Ignore malformed entries while preserving every valid user model.
      }
    }
    return models
  } catch {
    return []
  }
}

export function listUserModelSummaries(): UserModelSummary[] {
  return getUserModelConfigs().map(({ apiKey, ...model }) => ({
    ...model,
    hasApiKey: Boolean(apiKey)
  }))
}

export function saveUserModelConfig(value: unknown, previousId?: unknown): UserModelSummary[] {
  const model = normalizeModel(value)
  const oldId = typeof previousId === 'string' ? previousId.trim() : ''
  const models = getUserModelConfigs()
  const existing = models.find((candidate) => candidate.id === (oldId || model.id))
  const nextModel =
    !model.apiKey && existing?.apiKey ? { ...model, apiKey: existing.apiKey } : model
  const duplicate = models.find(
    (candidate) => candidate.id === nextModel.id && candidate.id !== existing?.id
  )
  if (duplicate) throw new Error(`A model with ID ${nextModel.id} already exists`)
  const next = existing
    ? models.map((candidate) => (candidate.id === existing.id ? nextModel : candidate))
    : [...models, nextModel]
  setStoredSetting(MODELS_SETTING_KEY, serializeModels(next))
  const selected = getStoredSetting('model')?.trim()
  if (!selected || selected === oldId) setStoredSetting('model', nextModel.id)
  return listUserModelSummaries()
}

export function deleteUserModelConfig(modelId: unknown): UserModelSummary[] {
  const id = requiredString(modelId, 'Model ID')
  const next = getUserModelConfigs().filter((model) => model.id !== id)
  setStoredSetting(MODELS_SETTING_KEY, serializeModels(next))
  if (getStoredSetting('model')?.trim() === id) {
    if (next[0]) setStoredSetting('model', next[0].id)
    else deleteStoredSetting('model')
  }
  return listUserModelSummaries()
}

export function resolveUserModelConfig(modelId?: string): UserModelConfig {
  const models = getUserModelConfigs()
  const selectedId = modelId?.trim() || getStoredSetting('model')?.trim()
  const model = selectedId ? models.find((candidate) => candidate.id === selectedId) : models[0]
  if (!model) {
    throw new Error('No LLM model is configured. Add one in Settings > Models.')
  }
  return model
}
