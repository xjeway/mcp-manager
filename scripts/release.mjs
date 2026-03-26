import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const files = {
  packageJson: path.join(repoRoot, 'package.json'),
  packageLock: path.join(repoRoot, 'package-lock.json'),
  tauriConfig: path.join(repoRoot, 'src-tauri', 'tauri.conf.json'),
  cargoToml: path.join(repoRoot, 'src-tauri', 'Cargo.toml'),
}

const semverPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`)
}

function readText(filePath) {
  return fs.readFileSync(filePath, 'utf8')
}

function writeText(filePath, value) {
  fs.writeFileSync(filePath, value)
}

function readVersions() {
  const packageJson = readJson(files.packageJson)
  const packageLock = readJson(files.packageLock)
  const tauriConfig = readJson(files.tauriConfig)
  const cargoToml = readText(files.cargoToml)
  const cargoVersionMatch = cargoToml.match(/^version = "([^"]+)"$/m)

  if (!cargoVersionMatch) {
    throw new Error('Unable to find version in src-tauri/Cargo.toml')
  }

  return {
    packageJson: packageJson.version,
    packageLock: packageLock.version,
    packageLockRoot: packageLock.packages?.['']?.version,
    tauriConfig: tauriConfig.version,
    cargoToml: cargoVersionMatch[1],
  }
}

function ensureVersion(version) {
  if (!semverPattern.test(version)) {
    throw new Error(`Invalid semver version: ${version}`)
  }
}

function ensureVersionsAligned(expectedVersion) {
  const versions = readVersions()
  const mismatchedEntries = Object.entries(versions).filter(([, value]) => value !== expectedVersion)

  if (mismatchedEntries.length > 0) {
    const details = mismatchedEntries.map(([name, value]) => `${name}=${value}`).join(', ')
    throw new Error(`Version mismatch detected. Expected ${expectedVersion}; found ${details}`)
  }
}

function updateVersion(version) {
  ensureVersion(version)

  const packageJson = readJson(files.packageJson)
  packageJson.version = version
  writeJson(files.packageJson, packageJson)

  const packageLock = readJson(files.packageLock)
  packageLock.version = version
  if (packageLock.packages?.['']) {
    packageLock.packages[''].version = version
  }
  writeJson(files.packageLock, packageLock)

  const tauriConfig = readJson(files.tauriConfig)
  tauriConfig.version = version
  writeJson(files.tauriConfig, tauriConfig)

  const cargoToml = readText(files.cargoToml)
  const nextCargoToml = cargoToml.replace(/^version = "([^"]+)"$/m, `version = "${version}"`)
  writeText(files.cargoToml, nextCargoToml)
}

function getGitStatus() {
  return execFileSync('git', ['status', '--short'], {
    cwd: repoRoot,
    encoding: 'utf8',
  }).trim()
}

function tagExists(tagName) {
  try {
    execFileSync('git', ['rev-parse', '-q', '--verify', `refs/tags/${tagName}`], {
      cwd: repoRoot,
      stdio: 'ignore',
    })
    return true
  } catch {
    return false
  }
}

function createTag(version) {
  ensureVersion(version)
  ensureVersionsAligned(version)

  const status = getGitStatus()
  if (status) {
    throw new Error('Git working tree is not clean. Commit version changes before creating a release tag.')
  }

  const tagName = `v${version}`
  if (tagExists(tagName)) {
    throw new Error(`Git tag already exists: ${tagName}`)
  }

  execFileSync('git', ['tag', '-a', tagName, '-m', `Release ${tagName}`], {
    cwd: repoRoot,
    stdio: 'inherit',
  })

  console.log(`Created tag ${tagName}`)
  console.log(`Next: git push origin ${tagName}`)
}

function printCurrent() {
  const versions = readVersions()
  console.log(JSON.stringify(versions, null, 2))
}

function printUsage() {
  console.error('Usage:')
  console.error('  node scripts/release.mjs current')
  console.error('  node scripts/release.mjs sync <version>')
  console.error('  node scripts/release.mjs tag <version>')
  console.error('  node scripts/release.mjs verify <version>')
}

function main() {
  const [command, version] = process.argv.slice(2)

  try {
    switch (command) {
      case 'current':
        printCurrent()
        return
      case 'sync':
        if (!version) {
          throw new Error('Missing version for sync command')
        }
        updateVersion(version)
        console.log(`Synchronized release version to ${version}`)
        return
      case 'verify':
        if (!version) {
          throw new Error('Missing version for verify command')
        }
        ensureVersion(version)
        ensureVersionsAligned(version)
        console.log(`Release version ${version} is aligned`)
        return
      case 'tag':
        if (!version) {
          throw new Error('Missing version for tag command')
        }
        createTag(version)
        return
      default:
        printUsage()
        process.exitCode = 1
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exitCode = 1
  }
}

main()
