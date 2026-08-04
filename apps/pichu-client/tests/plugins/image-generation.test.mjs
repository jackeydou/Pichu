import assert from 'node:assert/strict'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import test from 'node:test'
import { fileURLToPath, pathToFileURL } from 'node:url'

const testDir = dirname(fileURLToPath(import.meta.url))

async function loadImageGenerationForTest() {
  const moduleDir = mkdtempSync(join(testDir, 'image-generation-test-'))
  const modelConfigStorePath = join(moduleDir, 'model-config-store.ts')
  const imageGenerationPath = join(moduleDir, 'image-generation-under-test.ts')
  const sourcePath = new URL('../../src/main/tools/image-generation.ts', import.meta.url)
  const source = readFileSync(sourcePath, 'utf8').replace(
    '../stores/model-config-store.js',
    './model-config-store.ts'
  )

  writeFileSync(
    modelConfigStorePath,
    "export function resolveUserModelConfig() { throw new Error('not configured') }\n",
    'utf8'
  )
  writeFileSync(imageGenerationPath, source, 'utf8')

  try {
    return await import(`${pathToFileURL(imageGenerationPath).href}?ts=${Date.now()}`)
  } finally {
    rmSync(moduleDir, { recursive: true, force: true })
  }
}

test('normalizes image generation size aliases to mainstream high-resolution presets', async () => {
  const { normalizeImageGenerationSize } = await loadImageGenerationForTest()

  assert.equal(normalizeImageGenerationSize(undefined), 'auto')
  assert.equal(normalizeImageGenerationSize('auto'), 'auto')
  assert.equal(normalizeImageGenerationSize('square'), '2048x2048')
  assert.equal(normalizeImageGenerationSize('landscape'), '3840x2160')
  assert.equal(normalizeImageGenerationSize('portrait'), '2160x3840')
})

test('accepts valid flexible image generation sizes', async () => {
  const { normalizeImageGenerationSize } = await loadImageGenerationForTest()

  assert.equal(normalizeImageGenerationSize('1024x1024'), '1024x1024')
  assert.equal(normalizeImageGenerationSize('2048x2048'), '2048x2048')
  assert.equal(normalizeImageGenerationSize('3840x2160'), '3840x2160')
  assert.equal(normalizeImageGenerationSize('2160x3840'), '2160x3840')
})

test('rejects invalid image generation sizes', async () => {
  const { normalizeImageGenerationSize } = await loadImageGenerationForTest()

  assert.throws(() => normalizeImageGenerationSize('1000x1000'), /multiples of 16px/)
  assert.throws(() => normalizeImageGenerationSize('4096x2160'), /maximum edge length/)
  assert.throws(() => normalizeImageGenerationSize('3840x1024'), /ratio/)
  assert.throws(() => normalizeImageGenerationSize('512x512'), /total pixels/)
})
