import { describe, expect, it } from 'vitest'

async function loadResolver() {
  try {
    return await import('./release-args.mjs')
  } catch {
    return null
  }
}

describe('resolveReleaseArgs', () => {
  it('keeps msi for stable Windows x64 releases', async () => {
    const resolver = await loadResolver()

    expect(resolver).not.toBeNull()
    expect(
      resolver.resolveReleaseArgs({
        target: 'x86_64-pc-windows-msvc',
        refName: 'v0.1.2',
      }),
    ).toBe('--target x86_64-pc-windows-msvc --bundles nsis,msi')
  })

  it('skips msi for prerelease Windows x64 releases', async () => {
    const resolver = await loadResolver()

    expect(resolver).not.toBeNull()
    expect(
      resolver.resolveReleaseArgs({
        target: 'x86_64-pc-windows-msvc',
        refName: 'v0.1.2-rc.1',
      }),
    ).toBe('--target x86_64-pc-windows-msvc --bundles nsis')
  })

  it('keeps non-x64 Windows bundles unchanged', async () => {
    const resolver = await loadResolver()

    expect(resolver).not.toBeNull()
    expect(
      resolver.resolveReleaseArgs({
        target: 'aarch64-pc-windows-msvc',
        refName: 'v0.1.2-rc.1',
      }),
    ).toBe('--target aarch64-pc-windows-msvc --bundles nsis')
  })

  it('keeps Linux bundles unchanged', async () => {
    const resolver = await loadResolver()

    expect(resolver).not.toBeNull()
    expect(
      resolver.resolveReleaseArgs({
        target: 'x86_64-unknown-linux-gnu',
        refName: 'v0.1.2-rc.1',
      }),
    ).toBe('--target x86_64-unknown-linux-gnu --bundles appimage,deb,rpm')
  })
})
