import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

import { CLI_BIN_NAME, NPM_MAIN_PACKAGE, cliBinaryName, cliTargets, npmPlatformPackageName } from './cli-targets.mjs'
import { isPrereleaseRef } from './release-args.mjs'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const templateDir = path.join(repoRoot, 'crates', 'mcp-manager-cli', 'npm')

const description =
  'Add, list and remove MCP servers across Claude Code, Codex, Cursor, VS Code and more'

// npm checks this against the repository that trusted publishing runs from.
const sharedFields = {
  license: 'MIT',
  homepage: 'https://github.com/xjeway/mcp-manager#readme',
  repository: {
    type: 'git',
    url: 'git+https://github.com/xjeway/mcp-manager.git',
  },
}

const semverPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/

export function parseVersionFromTag(tagName) {
  const version = String(tagName).replace(/^v/, '')
  if (!semverPattern.test(version)) {
    throw new Error(`npm publishing requires a semver tag: ${tagName}`)
  }
  return version
}

export function npmDistTag(version) {
  return isPrereleaseRef(version) ? 'next' : 'latest'
}

export function renderPlatformPackageJson(entry, version) {
  return {
    name: npmPlatformPackageName(entry),
    version,
    description: `The ${entry.npm.os}-${entry.npm.cpu} binary for mcpmgr`,
    ...sharedFields,
    os: [entry.npm.os],
    cpu: [entry.npm.cpu],
    files: ['bin'],
    preferUnplugged: true,
  }
}

export function renderMainPackageJson(version) {
  return {
    name: NPM_MAIN_PACKAGE,
    version,
    description,
    ...sharedFields,
    keywords: ['mcp', 'model-context-protocol', 'claude-code', 'codex', 'cursor', 'cli'],
    bin: { [CLI_BIN_NAME]: `bin/${CLI_BIN_NAME}.js` },
    files: ['bin'],
    engines: { node: '>=18' },
    optionalDependencies: Object.fromEntries(
      cliTargets.map((entry) => [npmPlatformPackageName(entry), version]),
    ),
  }
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`)
}

function copyFile(from, to, mode) {
  fs.mkdirSync(path.dirname(to), { recursive: true })
  fs.copyFileSync(from, to)
  if (mode) {
    fs.chmodSync(to, mode)
  }
}

// Platform packages come first: the main package lists them as dependencies,
// so they have to be on the registry before it is.
export function packageDirs(outDir) {
  return [
    ...cliTargets.map((entry) => path.join(outDir, entry.target)),
    path.join(outDir, 'cli'),
  ]
}

// `binariesDir` holds one `mcpmgr-<target>/` folder per build, as
// actions/download-artifact lays out the release job's artifacts.
export function buildPackages({ version, binariesDir, outDir }) {
  fs.rmSync(outDir, { recursive: true, force: true })

  for (const entry of cliTargets) {
    const binaryName = cliBinaryName(entry.target)
    const source = path.join(binariesDir, `${CLI_BIN_NAME}-${entry.target}`, binaryName)
    if (!fs.existsSync(source)) {
      throw new Error(`Missing ${entry.target} binary: ${source}`)
    }

    const dir = path.join(outDir, entry.target)
    copyFile(source, path.join(dir, 'bin', binaryName), 0o755)
    copyFile(path.join(repoRoot, 'LICENSE'), path.join(dir, 'LICENSE'))
    writeJson(path.join(dir, 'package.json'), renderPlatformPackageJson(entry, version))
  }

  const mainDir = path.join(outDir, 'cli')
  const launcher = `${CLI_BIN_NAME}.js`
  copyFile(path.join(templateDir, 'bin', launcher), path.join(mainDir, 'bin', launcher), 0o755)
  copyFile(path.join(templateDir, 'README.md'), path.join(mainDir, 'README.md'))
  copyFile(path.join(repoRoot, 'LICENSE'), path.join(mainDir, 'LICENSE'))
  writeJson(path.join(mainDir, 'package.json'), renderMainPackageJson(version))

  return packageDirs(outDir)
}

// Trusted publishing can only be set up for a package that already exists, so
// each name is claimed once by hand with an empty 0.0.0 before the first release.
export function buildPlaceholders({ outDir, skip = () => false }) {
  fs.rmSync(outDir, { recursive: true, force: true })
  const names = [...cliTargets.map((entry) => npmPlatformPackageName(entry)), NPM_MAIN_PACKAGE].filter(
    (name) => !skip(name),
  )

  return names.map((name) => {
    const dir = path.join(outDir, name.replace(/^@[^/]+\//, ''))
    fs.mkdirSync(dir, { recursive: true })
    writeJson(path.join(dir, 'package.json'), {
      name,
      version: '0.0.0',
      description: `Placeholder for ${name}; the first release replaces it.`,
      ...sharedFields,
    })
    return dir
  })
}

function publishedVersion(spec) {
  try {
    return execFileSync('npm', ['view', spec, 'version'], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim()
  } catch {
    return ''
  }
}

function isPublished(name, version) {
  return publishedVersion(`${name}@${version}`) === version
}

// Skips versions already on the registry, so a rerun after a partial failure
// finishes the remaining packages instead of failing on the first one.
function publishPackages({ outDir, dryRun }) {
  for (const dir of packageDirs(outDir)) {
    const { name, version } = JSON.parse(fs.readFileSync(path.join(dir, 'package.json'), 'utf8'))
    if (isPublished(name, version)) {
      console.log(`${name}@${version} is already published`)
      continue
    }

    const args = ['publish', dir, '--access', 'public', '--tag', npmDistTag(version)]
    if (dryRun) {
      args.push('--dry-run')
    }
    console.log(`npm ${args.join(' ')}`)
    execFileSync('npm', args, { stdio: 'inherit' })
  }
}

export function parseArgs(argv) {
  const args = { command: argv[2], tag: null, binariesDir: null, outDir: 'npm-dist', dryRun: false }

  for (let index = 3; index < argv.length; index += 1) {
    const arg = argv[index]
    const value = argv[index + 1]
    if (arg === '--tag') {
      args.tag = value
      index += 1
    } else if (arg === '--binaries') {
      args.binariesDir = value
      index += 1
    } else if (arg === '--out') {
      args.outDir = value
      index += 1
    } else if (arg === '--dry-run') {
      args.dryRun = true
    } else {
      throw new Error(`Unknown argument: ${arg}`)
    }
  }

  return args
}

function printUsage() {
  console.error('Usage:')
  console.error('  node scripts/npm-packages.mjs build --tag vX.Y.Z --binaries DIR [--out DIR]')
  console.error('  node scripts/npm-packages.mjs publish [--out DIR] [--dry-run]')
  console.error('  node scripts/npm-packages.mjs bootstrap [--out DIR]')
}

function main() {
  const args = parseArgs(process.argv)

  if (args.command === 'build') {
    if (!args.tag || !args.binariesDir) {
      printUsage()
      process.exitCode = 1
      return
    }
    const version = parseVersionFromTag(args.tag)
    const dirs = buildPackages({ version, binariesDir: args.binariesDir, outDir: args.outDir })
    console.log(`Built ${dirs.length} npm packages for ${version} in ${args.outDir}`)
    return
  }

  if (args.command === 'publish') {
    publishPackages(args)
    return
  }

  if (args.command === 'bootstrap') {
    const dirs = buildPlaceholders({ ...args, skip: (name) => publishedVersion(name) !== '' })
    if (dirs.length === 0) {
      console.log('Every package name is already on npm; nothing to claim.')
    }
    for (const dir of dirs) {
      console.log(`npm publish ${dir} --access public`)
    }
    return
  }

  printUsage()
  process.exitCode = 1
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main()
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exitCode = 1
  }
}
