export const MODEL_API_SPECS = [
  'openai-responses',
  'openai-completions',
  'anthropic-messages',
  'google-generative-ai'
] as const

export type ModelApiSpec = (typeof MODEL_API_SPECS)[number]

export type UserModelConfig = {
  id: string
  name: string
  api: ModelApiSpec
  baseUrl: string
  apiKey: string
  contextWindow: number
  maxTokens: number
  reasoning: boolean
  supportsImages: boolean
}

export type UserModelSummary = Omit<UserModelConfig, 'apiKey'> & {
  hasApiKey: boolean
}

export function isModelApiSpec(value: unknown): value is ModelApiSpec {
  return typeof value === 'string' && MODEL_API_SPECS.includes(value as ModelApiSpec)
}
