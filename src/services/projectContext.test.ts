// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from 'vitest'
import { configForProject, mergeProjectChange, projectContextChanged, placementInProject, readRecentProjects, saveProjectSelection, readProjectSelection } from './projectContext'
import { SUPPORTED_APPS, type MCPConfig } from '../types/config'

const workspace = { root: '/projects/a', placementPaths: {} }
function fixture(): MCPConfig {
  return { version: 1, servers: [{id:'shared',name:'Shared',enabled:true,transport:{type:'stdio'},command:{program:'node',args:[],env:{}},apps:Object.fromEntries(SUPPORTED_APPS.map((app) => [app,false])) as MCPConfig['servers'][number]['apps'],placements:[
    {app:'cursor',scope:'workspace',path:'/projects/a/.cursor/mcp.json',enabled:true,managed:true},
    {app:'cursor',scope:'workspace',path:'/projects/b/.cursor/mcp.json',enabled:true,managed:true},
  ]}] }
}

describe('project boundaries', () => {
  beforeEach(() => window.localStorage.clear())
  it('shows only selected placements and preserves other projects on deletion', () => {
    const previous=fixture();const view=configForProject(previous,workspace)
    expect(view.servers[0].placements).toHaveLength(1)
    expect(previous.servers[0].placements).toHaveLength(2)
    const next=mergeProjectChange(previous,{version:1,servers:[]},workspace)
    expect(next.servers[0].placements?.map((p)=>p.path)).toEqual(['/projects/b/.cursor/mcp.json'])
    expect(configForProject(next,workspace).servers).toHaveLength(0)
  })
  it('makes local disabling preserve the shared enabled state and foreign placements', () => {
    const previous=fixture();const view=configForProject(previous,workspace);view.servers[0].enabled=false
    const next=mergeProjectChange(previous,view,workspace)
    expect(next.servers[0].enabled).toBe(true)
    expect(next.servers[0].placements?.map((p)=>p.enabled)).toEqual([false,true])
  })
  it('recognizes backend-resolved aliases and ordinary/extended Windows paths', () => {
    const p=fixture().servers[0].placements![0]
    expect(placementInProject({...p,path:'/alias/repo/.cursor/mcp.json'}, {...workspace,contextPaths:['/alias/repo/.cursor/mcp.json']})).toBe(true)
    expect(placementInProject({...p,path:'C:/Work/A/.cursor/mcp.json'}, {root:'\\\\?\\C:\\Work\\A',placementPaths:{}})).toBe(true)
    expect(placementInProject({...p,path:'/projects/ab/.cursor/mcp.json'},workspace)).toBe(false)
    expect(configForProject(fixture(),{root:'',placementPaths:{}}).servers).toHaveLength(0)
  })
  it('uses authoritative native ownership for escapes, nested projects and missing owners', () => {
    const p = fixture().servers[0].placements![0]
    const native = { ...workspace, contextPaths: [p.path!] }
    expect(placementInProject(p, native)).toBe(true)
    expect(placementInProject({ ...p, path: '/projects/a/nested/.cursor/mcp.json' }, native)).toBe(false)
    expect(placementInProject({ ...p, path: '/projects/a/../b/.cursor/mcp.json' }, native)).toBe(false)
    expect(placementInProject({ ...p, path: '/projects/a/linked-outside/mcp.json' }, native)).toBe(false)
    expect(placementInProject({ ...p, path: undefined }, native)).toBe(false)
  })
  it('detects newly resolved aliases before merging a change made from a stale view', () => {
    const previous = fixture()
    previous.servers[0].placements = [{ ...previous.servers[0].placements![0], path: '/alias/.cursor/mcp.json' }]
    const stale = { ...workspace, contextPaths: [] }
    const fresh = { ...workspace, contextPaths: ['/alias/.cursor/mcp.json'] }
    expect(configForProject(previous, stale).servers).toHaveLength(0)
    expect(configForProject(previous, fresh).servers).toHaveLength(1)
    expect(projectContextChanged(stale, fresh)).toBe(true)
    expect(mergeProjectChange(previous, { version: 1, servers: [] }, stale).servers[0].placements).toEqual(previous.servers[0].placements)
  })
  it('remembers explicit computer-only selection and bounds recent projects', () => {
    expect(readProjectSelection()).toBeUndefined()
    for(let i=0;i<7;i++) saveProjectSelection(`/projects/${i}`)
    saveProjectSelection('/projects/4')
    expect(readRecentProjects()).toEqual(['/projects/4','/projects/6','/projects/5','/projects/3','/projects/2'])
    saveProjectSelection('');expect(readProjectSelection()).toBe('');expect(readRecentProjects()).toHaveLength(5)
  })
})
