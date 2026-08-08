import assert from 'node:assert/strict'
import test from 'node:test'
import { splitDevArgs } from '../../scripts/dev.mjs'
import { parseDataRootArg, parseDevNameArg } from '../../src/shared/startup-args.ts'

test('parseDataRootArg reads namespaced inline values', () => {
  assert.deepEqual(parseDataRootArg(['electron', '--pichu-data-root=/tmp/pichu']), {
    name: '--pichu-data-root',
    value: '/tmp/pichu'
  })
})

test('parseDataRootArg reads following values', () => {
  assert.deepEqual(parseDataRootArg(['electron', '--data-root', '~/pichu-dev']), {
    name: '--data-root',
    value: '~/pichu-dev'
  })
})

test('parseDataRootArg ignores missing values', () => {
  assert.equal(parseDataRootArg(['electron', '--pichu-data-root']), null)
  assert.equal(parseDataRootArg(['electron', '--pichu-data-root', '--another-flag']), null)
  assert.equal(parseDataRootArg(['electron', '--data-root=']), null)
})

test('parseDevNameArg reads inline and following values', () => {
  assert.deepEqual(parseDevNameArg(['electron', '--pichu-dev-name=Search QA']), {
    name: '--pichu-dev-name',
    value: 'Search QA'
  })
  assert.deepEqual(parseDevNameArg(['electron', '--dev-name', 'staging test']), {
    name: '--dev-name',
    value: 'staging test'
  })
})

test('parseDevNameArg ignores missing values', () => {
  assert.equal(parseDevNameArg(['electron', '--pichu-dev-name']), null)
  assert.equal(parseDevNameArg(['electron', '--dev-name=']), null)
})

test('splitDevArgs passes app startup args through electron-vite separator', () => {
  assert.deepEqual(splitDevArgs(['--pichu-dev-name', 'Search QA']), [
    'dev',
    '--',
    '--pichu-dev-name',
    'Search QA'
  ])
  assert.deepEqual(splitDevArgs(['--inspect', '--pichu-data-root=~/.pichu-dev/search']), [
    'dev',
    '--inspect',
    '--',
    '--pichu-data-root=~/.pichu-dev/search'
  ])
})

test('splitDevArgs enables react scan through electron-vite mode', () => {
  assert.deepEqual(splitDevArgs(['--react-scan', '--pichu-dev-name', 'Search QA']), [
    'dev',
    '--mode',
    'react-scan',
    '--',
    '--pichu-dev-name',
    'Search QA'
  ])
})

test('splitDevArgs ignores explicit pnpm separators', () => {
  assert.deepEqual(splitDevArgs(['--', '--pichu-dev-name', 'Search QA']), [
    'dev',
    '--',
    '--pichu-dev-name',
    'Search QA'
  ])
  assert.deepEqual(splitDevArgs(['--']), ['dev'])
})
