export const IMAGE_GENERATION_MODEL = 'gpt-image-2' as const

export type ImageGenerationConfigStatus = {
  model: typeof IMAGE_GENERATION_MODEL
  hasApiKey: boolean
}
