import { SUPPORTED_APPS, type MCPServer } from '../types/config'
import type { InstallInput, InstallOption, MarketplaceEntry } from '../types/marketplace'

/** User-entered values for an option's inputs, keyed by `InstallInput.key`. */
export type InputValues = Record<string, string>

const PLACEHOLDER = /\{([A-Za-z0-9_.-]+)\}/g
const SECRET_MASK = '••••'

export function serverIdFor(entry: MarketplaceEntry, existingIds: string[]): string {
  const base =
    (entry.id.split('/').pop() ?? entry.id)
      .toLowerCase()
      .replace(/[^a-z0-9._-]+/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '') || 'mcp-server'
  if (!existingIds.includes(base)) {
    return base
  }
  let suffix = 2
  while (existingIds.includes(`${base}-${suffix}`)) {
    suffix += 1
  }
  return `${base}-${suffix}`
}

/** Declared npm/PyPI first (what most users expect), then any runnable option. */
export function defaultOptionIndex(options: InstallOption[]): number {
  const preferred = options.findIndex(
    (option) => option.kind === 'stdio' && !option.inferred && (option.packageType === 'npm' || option.packageType === 'pypi'),
  )
  if (preferred >= 0) {
    return preferred
  }
  const runnable = options.findIndex((option) => option.kind !== 'unsupported')
  return runnable >= 0 ? runnable : 0
}

function inputsOf(option: InstallOption): InstallInput[] {
  return option.kind === 'unsupported' ? [] : option.inputs
}

function placeholderKeys(value: string): string[] {
  return [...value.matchAll(PLACEHOLDER)].map((match) => match[1])
}

/**
 * Headers cannot be written to client configs yet, so inputs that only feed headers are
 * shown for reference and never block adding the server.
 */
function headerOnlyKeys(option: InstallOption): Set<string> {
  if (option.kind !== 'http') {
    return new Set()
  }
  const inUrl = new Set(placeholderKeys(option.url))
  return new Set(
    Object.values(option.headers)
      .flatMap(placeholderKeys)
      .filter((key) => !inUrl.has(key)),
  )
}

/** Inputs the user fills in for this option (header-only inputs excluded). */
export function editableInputs(option: InstallOption): InstallInput[] {
  const headerOnly = headerOnlyKeys(option)
  return inputsOf(option).filter((item) => !headerOnly.has(item.key))
}

function valueFor(item: InstallInput | undefined, values: InputValues): string {
  if (!item) {
    return ''
  }
  const entered = values[item.key]?.trim() ?? ''
  return entered || item.defaultValue || ''
}

export function missingRequiredInputs(option: InstallOption, values: InputValues): InstallInput[] {
  return editableInputs(option).filter((item) => item.required && !valueFor(item, values))
}

export function manualHeaders(option: InstallOption): Array<{ name: string; value: string }> {
  return option.kind === 'http' ? Object.entries(option.headers).map(([name, value]) => ({ name, value })) : []
}

interface Resolved {
  text: string
  /** A placeholder had no value; the caller drops optional parts. */
  incomplete: boolean
}

function resolve(template: string, inputs: InstallInput[], values: InputValues, mask: boolean): Resolved {
  let incomplete = false
  const text = template.replace(PLACEHOLDER, (whole, key: string) => {
    const item = inputs.find((candidate) => candidate.key === key)
    const value = valueFor(item, values)
    if (!value) {
      incomplete = true
      return whole
    }
    return mask && item?.secret ? SECRET_MASK : value
  })
  return { text, incomplete }
}

/**
 * Flattens argument groups, dropping any group whose placeholder was left empty: those
 * are optional flags, and required ones are enforced by `missingRequiredInputs`.
 */
function resolveArgs(option: Extract<InstallOption, { kind: 'stdio' }>, values: InputValues, mask: boolean): string[] {
  return option.argGroups.flatMap((group) => {
    const resolved = group.map((part) => resolve(part, option.inputs, values, mask))
    const required = group
      .flatMap(placeholderKeys)
      .some((key) => option.inputs.find((item) => item.key === key)?.required)
    return resolved.some((part) => part.incomplete) && !required ? [] : resolved.map((part) => part.text)
  })
}

function resolveEnv(option: Extract<InstallOption, { kind: 'stdio' }>, values: InputValues): Record<string, string> {
  return Object.fromEntries(
    Object.entries(option.env).flatMap(([name, template]) => {
      const resolved = resolve(template, option.inputs, values, false)
      return resolved.incomplete ? [] : [[name, resolved.text]]
    }),
  )
}

export function buildServerFromOption(
  entry: MarketplaceEntry,
  option: InstallOption,
  values: InputValues,
  existingIds: string[],
): MCPServer {
  const base: MCPServer = {
    id: serverIdFor(entry, existingIds),
    name: entry.title || entry.id,
    enabled: true,
    description: entry.description || undefined,
    homepage: entry.repositoryUrl ?? entry.websiteUrl ?? undefined,
    transport: { type: 'stdio' },
    apps: Object.fromEntries(SUPPORTED_APPS.map((app) => [app, false])) as MCPServer['apps'],
  }

  if (option.kind === 'http') {
    return { ...base, transport: { type: 'http', url: resolve(option.url, option.inputs, values, false).text } }
  }
  if (option.kind === 'stdio') {
    return {
      ...base,
      command: { program: option.program, args: resolveArgs(option, values, false), env: resolveEnv(option, values) },
    }
  }
  return base
}

function quote(arg: string): string {
  return /[\s'"]/.test(arg) ? `'${arg.replace(/'/g, `'\\''`)}'` : arg
}

/** What will run, as a shell-style line; secret values are masked. */
export function commandPreview(option: InstallOption, values: InputValues): string {
  if (option.kind === 'http') {
    return resolve(option.url, option.inputs, values, true).text
  }
  if (option.kind === 'stdio') {
    return [option.program, ...resolveArgs(option, values, true)].map(quote).join(' ')
  }
  return option.identifier
}

function normalizeRepositoryUrl(url: string): string {
  return url
    .trim()
    .toLowerCase()
    .replace(/\.git$/, '')
    .replace(/\/+$/, '')
}

export function isEntryAdded(entry: MarketplaceEntry, servers: MCPServer[]): boolean {
  const repository = entry.repositoryUrl ? normalizeRepositoryUrl(entry.repositoryUrl) : null
  const id = serverIdFor(entry, [])
  return servers.some(
    (server) =>
      server.id === id || (repository !== null && !!server.homepage && normalizeRepositoryUrl(server.homepage) === repository),
  )
}
