import fs from 'node:fs/promises'
import path from 'node:path'
import url from 'node:url'
import * as esbuild from 'esbuild'

const root = path.dirname(path.dirname(url.fileURLToPath(import.meta.url)))
const dist = path.join(root, 'dist')

await fs.rm(dist, { recursive: true, force: true })
await fs.mkdir(path.join(dist, 'assets'), { recursive: true })

const nodeEnv = process.env.NODE_ENV ?? 'development'
const isProd = nodeEnv === 'production'

await esbuild.build({
  entryPoints: [path.join(root, 'src', 'main.tsx')],
  bundle: true,
  format: 'esm',
  splitting: false,
  sourcemap: isProd ? false : 'inline',
  minify: isProd,
  target: ['es2022'],
  jsx: 'automatic',
  outfile: path.join(dist, 'assets', 'app.js'),
  loader: {
    '.css': 'css'
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(nodeEnv)
  }
})

await fs.copyFile(path.join(root, 'index.html'), path.join(dist, 'index.html'))
