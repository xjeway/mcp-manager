import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const releaseArgsByTarget = Object.freeze({
  'aarch64-apple-darwin': '--target aarch64-apple-darwin --bundles app,dmg',
  'x86_64-apple-darwin': '--target x86_64-apple-darwin --bundles app,dmg',
  'x86_64-unknown-linux-gnu': '--target x86_64-unknown-linux-gnu --bundles appimage,deb,rpm',
  'aarch64-unknown-linux-gnu': '--target aarch64-unknown-linux-gnu --bundles appimage,deb,rpm',
  'x86_64-pc-windows-msvc': '--target x86_64-pc-windows-msvc --bundles nsis,msi',
  'aarch64-pc-windows-msvc': '--target aarch64-pc-windows-msvc --bundles nsis',
})

export function isPrereleaseRef(refName) {
  return /^v?\d+\.\d+\.\d+-/.test(String(refName).trim())
}

export function resolveReleaseArgs({ target, refName }) {
  const defaultArgs = releaseArgsByTarget[target]

  if (!defaultArgs) {
    throw new Error(`Unsupported release target: ${target}`)
  }

  if (target === 'x86_64-pc-windows-msvc' && isPrereleaseRef(refName)) {
    return '--target x86_64-pc-windows-msvc --bundles nsis'
  }

  return defaultArgs
}

function printUsage() {
  console.error('Usage:')
  console.error('  node scripts/release-args.mjs <target> <git-ref-name>')
}

function main() {
  const [target, refName] = process.argv.slice(2)

  if (!target || !refName) {
    printUsage()
    process.exitCode = 1
    return
  }

  console.log(resolveReleaseArgs({ target, refName }))
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main()
}
