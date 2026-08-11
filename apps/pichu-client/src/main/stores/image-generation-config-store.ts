import { safeStorage } from 'electron'
import {
  IMAGE_GENERATION_MODEL,
  type ImageGenerationConfigStatus
} from '../../shared/image-generation-config.js'
import { deleteStoredSetting, getStoredSetting, setStoredSetting } from './settings-store.js'

const IMAGE_GENERATION_API_KEY_SETTING = 'imageGenerationApiKey'

export function getImageGenerationApiKey(): string | undefined {
  const encryptedApiKey = getStoredSetting(IMAGE_GENERATION_API_KEY_SETTING)
  if (!encryptedApiKey) return undefined
  try {
    return safeStorage.decryptString(Buffer.from(encryptedApiKey, 'base64')).trim() || undefined
  } catch {
    return undefined
  }
}

export function hasImageGenerationApiKey(): boolean {
  return Boolean(getImageGenerationApiKey())
}

export function getImageGenerationConfigStatus(): ImageGenerationConfigStatus {
  return {
    model: IMAGE_GENERATION_MODEL,
    hasApiKey: hasImageGenerationApiKey()
  }
}

export function saveImageGenerationApiKey(value: unknown): ImageGenerationConfigStatus {
  if (typeof value !== 'string' || !value.trim()) {
    throw new Error('Image generation API key is required')
  }
  const encryptedApiKey = safeStorage.encryptString(value.trim()).toString('base64')
  setStoredSetting(IMAGE_GENERATION_API_KEY_SETTING, encryptedApiKey)
  return getImageGenerationConfigStatus()
}

export function clearImageGenerationApiKey(): ImageGenerationConfigStatus {
  deleteStoredSetting(IMAGE_GENERATION_API_KEY_SETTING)
  return getImageGenerationConfigStatus()
}
