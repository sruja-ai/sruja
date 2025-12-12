import { useMemo, useState } from 'react';
import { Button, Input } from '@sruja/ui';
import { CheckCircle, AlertCircle, Info, Layout, FileCode } from 'lucide-react';
import { useArchitectureStore } from '../../stores/architectureStore';
import { useViewStore } from '../../stores/viewStore';
import { useUIStore } from '../../stores/uiStore';
import { useFeatureFlagsStore } from '../../stores/featureFlagsStore';
import type { ArchitectureJSON, SystemJSON, ContainerJSON } from '../../types';
import './GuidedBuilderPanel.css';

export function GuidedBuilderPanel() {
    const data = useArchitectureStore((s) => s.data);
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
    const isEditMode = useFeatureFlagsStore((s) => s.isEditMode);
    const focusedSystemId = useViewStore((s) => s.focusedSystemId);
    const focusedContainerId = useViewStore((s) => s.focusedContainerId);
    const drillDown = useViewStore((s) => s.drillDown);
    const setActiveTab = useUIStore((s) => s.setActiveTab);

    const systems = useMemo(() => data?.architecture.systems ?? [], [data]);
    const persons = useMemo(() => data?.architecture.persons ?? [], [data]);
    const allContainers = useMemo(() => systems.flatMap(s => s.containers || []), [systems]);
    const relations = useMemo(() => data?.architecture.relations ?? [], [data]);
    const requirements = useMemo(() => data?.architecture.requirements ?? [], [data]);
    const flows = useMemo(() => data?.architecture.flows ?? [], [data]);
    const scenarios = useMemo(() => data?.architecture.scenarios ?? [], [data]);
    const adrs = useMemo(() => data?.architecture.adrs ?? [], [data]);

    const [systemName, setSystemName] = useState('');
    const [personName, setPersonName] = useState('');
    const [containerName, setContainerName] = useState('');
    const [containerTech, setContainerTech] = useState('');
    const [containerExternal, setContainerExternal] = useState(false);
    const [containerParentId, setContainerParentId] = useState<string | ''>(focusedSystemId || systems[0]?.id || '');
    const [componentName, setComponentName] = useState('');
    const [componentParentId, setComponentParentId] = useState<string | ''>(focusedContainerId || allContainers[0]?.id || '');

    const slugify = (text: string) => text.toLowerCase().trim().replace(/[^a-z0-9_-]+/g, '-').replace(/^-+|-+$/g, '');

    const submitAddSystem = async () => {
        const id = slugify(systemName) || 'system';
        await updateArchitecture((arch: ArchitectureJSON) => {
            const systems = [...(arch.architecture?.systems || [])];
            if (!systems.find(s => s.id === id)) systems.push({ id, label: systemName, containers: [] } as SystemJSON);
            return { ...arch, architecture: { ...arch.architecture, systems } };
        });
        setSystemName('');
    };

    const submitAddPerson = async () => {
        const id = slugify(personName) || 'person';
        await updateArchitecture((arch: ArchitectureJSON) => {
            const persons = [...(arch.architecture?.persons || [])];
            if (!persons.find(p => p.id === id)) persons.push({ id, label: personName });
            return { ...arch, architecture: { ...arch.architecture, persons } };
        });
        setPersonName('');
    };

    const submitAddContainer = async () => {
        if (!containerParentId) return;
        const id = slugify(containerName) || 'container';
        await updateArchitecture((arch: ArchitectureJSON) => {
            const systems = (arch.architecture?.systems || []).map(s => {
                if (s.id !== containerParentId) return s;
                const containers = [...(s.containers || [])];
                const tags = containerExternal ? ['external'] : undefined;
                if (!containers.find(c => c.id === id)) containers.push({ id, label: containerName, technology: containerTech || undefined, tags, components: [] } as ContainerJSON);
                return { ...s, containers };
            });
            return { ...arch, architecture: { ...arch.architecture, systems } };
        });
        setContainerName('');
        setContainerTech('');
        setContainerExternal(false);
    };

    const submitAddComponent = async () => {
        if (!componentParentId) return;
        const id = slugify(componentName) || 'component';
        await updateArchitecture((arch: ArchitectureJSON) => {
            const systems = (arch.architecture?.systems || []).map(s => {
                const containers = (s.containers || []).map(c => {
                    if (c.id !== componentParentId) return c;
                    const components = [...(c.components || [])];
                    if (!components.find(co => co.id === id)) components.push({ id, label: componentName });
                    return { ...c, components };
                });
                return { ...s, containers };
            });
            return { ...arch, architecture: { ...arch.architecture, systems } };
        });
        setComponentName('');
    };

    const personIds = useMemo(() => new Set(persons.map(p => p.id)), [persons]);
    const systemIds = useMemo(() => new Set(systems.map(s => s.id)), [systems]);
    const containerIds = useMemo(() => new Set(allContainers.map(c => c.id)), [allContainers]);
    const componentIds = useMemo(() => new Set(allContainers.flatMap(c => c.components || []).map(co => co.id)), [allContainers]);

    const hasL1Relations = relations.some(r => (personIds.has(r.from) || systemIds.has(r.from)) && (personIds.has(r.to) || systemIds.has(r.to)));
    const hasContainerRelations = relations.some(r => containerIds.has(r.from) || containerIds.has(r.to));
    const hasComponentRelations = relations.some(r => componentIds.has(r.from) || componentIds.has(r.to));

    const l1Complete = persons.length > 0 && systems.length > 0 && hasL1Relations;


    const systemsWithDescriptions = systems.filter(s => !!s.description && s.description.trim().length > 0).length;
    const containersWithTech = allContainers.filter(c => !!c.technology && c.technology.trim().length > 0).length;
    const componentsAll = allContainers.flatMap(c => c.components || []);
    const componentsWithTech = componentsAll.filter(co => !!co.technology && co.technology.trim().length > 0).length;

    const reqTaggedTo = (ids: Set<string>) => requirements.filter(r => (r.tags || []).some(tag => ids.has(tag))).length;
    const l1ReqTagged = reqTaggedTo(new Set([...persons.map(p => p.id), ...systems.map(s => s.id)]));
    const l2ReqTagged = reqTaggedTo(new Set([...allContainers.map(c => c.id), ...systems.map(s => s.id)]));
    const l3ReqTagged = reqTaggedTo(new Set([...componentsAll.map(c => c.id), ...allContainers.map(c => c.id)]));

    const containerParentById = useMemo(() => {
        const m = new Map<string, string>();
        systems.forEach(s => (s.containers || []).forEach(c => m.set(c.id, s.id)));
        return m;
    }, [systems]);
    const componentParentById = useMemo(() => {
        const m = new Map<string, { sysId: string; containerId: string }>();
        systems.forEach(s => (s.containers || []).forEach(c => (c.components || []).forEach(co => m.set(co.id, { sysId: s.id, containerId: c.id }))));
        return m;
    }, [systems]);
    const containerHasRelations = (cid: string) => {
        const sysId = containerParentById.get(cid);
        const qualified = sysId ? `${sysId}.${cid}` : undefined;
        return relations.some(r => r.from === cid || r.to === cid || (qualified ? (r.from === qualified || r.to === qualified) : false));
    };
    const componentHasRelations = (coid: string) => {
        const parent = componentParentById.get(coid);
        const qualified = parent ? `${parent.sysId}.${parent.containerId}.${coid}` : undefined;
        return relations.some(r => r.from === coid || r.to === coid || (qualified ? (r.from === qualified || r.to === qualified) : false));
    };
    const containersMissingRelations = allContainers.filter(c => !containerHasRelations(c.id));
    const componentsMissingRelations = componentsAll.filter(co => !componentHasRelations(co.id));
    const containersWithRelations = allContainers.length - containersMissingRelations.length;
    const componentsWithRelations = componentsAll.length - componentsMissingRelations.length;

    const archDatastores = (data?.architecture.datastores || []).map(ds => ds.id);
    const systemDatastoresQualified = systems.flatMap(sys => (sys.datastores || []).map(ds => `${sys.id}.${ds.id}`));
    const datastoreIds = new Set<string>([...archDatastores, ...systemDatastoresQualified]);
    const sharedDatastores = Array.from(datastoreIds).filter(dsId => {
        const consumers = new Set<string>();
        relations.forEach(r => {
            if (r.to === dsId || r.from === dsId || r.to === dsId.split('.').pop() || r.from === dsId.split('.').pop()) {
                consumers.add(r.from);
                consumers.add(r.to);
            }
        });
        return consumers.size > 2;
    }).length;

    const layers = ['web', 'api', 'service', 'data', 'database'];
    const layerIndex = (name?: string) => name ? layers.indexOf(name) : -1;
    const resolveMetaLayer = (id: string): string | undefined => {
        // System
        const sys = systems.find(s => s.id === id);
        if (sys) return sys.metadata?.find(m => m.key === 'layer')?.value?.replace(/"/g, '').toLowerCase();
        // Container (qualified/unqualified)
        const parts = id.split('.');
        if (parts.length === 2) {
            const s = systems.find(x => x.id === parts[0]);
            const c = s?.containers?.find(y => y.id === parts[1]);
            return c?.metadata?.find(m => m.key === 'layer')?.value?.replace(/"/g, '').toLowerCase();
        }
        if (parts.length === 1) {
            const c = allContainers.find(y => y.id === parts[0]);
            return c?.metadata?.find(m => m.key === 'layer')?.value?.replace(/"/g, '').toLowerCase();
        }
        // Component
        if (parts.length === 3) {
            const s = systems.find(x => x.id === parts[0]);
            const c = s?.containers?.find(y => y.id === parts[1]);
            const co = c?.components?.find(z => z.id === parts[2]);
            return co?.metadata?.find(m => m.key === 'layer')?.value?.replace(/"/g, '').toLowerCase();
        }
        return undefined;
    };
    const layeringViolations = relations.filter(r => {
        const fromL = resolveMetaLayer(r.from);
        const toL = resolveMetaLayer(r.to);
        const fi = layerIndex(fromL);
        const ti = layerIndex(toL);
        return fi !== -1 && ti !== -1 && fi > ti;
    }).length;
    const layeringViolationDetails = relations
        .map(r => {
            const fromL = resolveMetaLayer(r.from);
            const toL = resolveMetaLayer(r.to);
            const fi = layerIndex(fromL);
            const ti = layerIndex(toL);
            return fi !== -1 && ti !== -1 && fi > ti ? { from: r.from, to: r.to, fromLayer: fromL, toLayer: toL } : null;
        })
        .filter(Boolean) as { from: string; to: string; fromLayer?: string; toLayer?: string }[];

    const [containerTargetMap, setContainerTargetMap] = useState<Record<string, string>>({});
    const [componentTargetMap, setComponentTargetMap] = useState<Record<string, string>>({});
    const [showAllContainerFixes, setShowAllContainerFixes] = useState(false);
    const [showAllComponentFixes, setShowAllComponentFixes] = useState(false);
    const [showLayeringDetails, setShowLayeringDetails] = useState(false);
    const [layeringFixTargetMap, setLayeringFixTargetMap] = useState<Record<string, string>>({});

    const getContainerTargets = (cid: string): { id: string; label: string }[] => {
        const sysId = containerParentById.get(cid);
        const sys = systems.find(s => s.id === sysId);
        const siblings = (sys?.containers || []).filter(c => c.id !== cid).map(c => ({ id: `${sysId}.${c.id}`, label: c.label || c.id }));
        const dss = (sys?.datastores || []).map(ds => ({ id: `${sysId}.${ds.id}`, label: ds.label || ds.id }));
        const targets = [...siblings, ...dss];
        return targets.length > 0 ? targets : (sysId ? [{ id: sysId, label: sys?.label || sysId }] : []);
    };

    const getComponentTargets = (coid: string): { id: string; label: string }[] => {
        const parent = componentParentById.get(coid);
        if (!parent) return [];
        const sys = systems.find(s => s.id === parent.sysId);
        const container = sys?.containers?.find(c => c.id === parent.containerId);
        const siblings = (container?.components || []).filter(co => co.id !== coid).map(co => ({ id: `${parent.sysId}.${parent.containerId}.${co.id}`, label: co.label || co.id }));
        const targets = [...siblings, { id: `${parent.sysId}.${parent.containerId}`, label: container?.label || parent.containerId }];
        return targets;
    };

    const getDownwardTargets = (fromId: string): { id: string; label: string }[] => {
        const fromLayer = resolveMetaLayer(fromId);
        const fi = layerIndex(fromLayer);
        if (fi === -1) return [];
        const parts = fromId.split('.');
        if (parts.length === 1) {
            const sys = systems.find(s => s.id === parts[0]);
            const conts = (sys?.containers || [])
                .map(c => ({ id: `${parts[0]}.${c.id}`, label: c.label || c.id }))
                .filter(t => layerIndex(resolveMetaLayer(t.id)) > fi);
            const dss = (sys?.datastores || [])
                .map(ds => ({ id: `${parts[0]}.${ds.id}`, label: ds.label || ds.id }))
                .filter(t => layerIndex(resolveMetaLayer(t.id)) > fi);
            return [...conts, ...dss];
        }
        if (parts.length === 3) {
            const sys = systems.find(s => s.id === parts[0]);
            const container = sys?.containers?.find(c => c.id === parts[1]);
            const comps = (container?.components || [])
                .map(co => ({ id: `${parts[0]}.${parts[1]}.${co.id}`, label: co.label || co.id }))
                .filter(t => layerIndex(resolveMetaLayer(t.id)) > fi);
            const cont = { id: `${parts[0]}.${parts[1]}`, label: container?.label || parts[1] };
            const contLayer = layerIndex(resolveMetaLayer(cont.id));
            const targets = [...comps, ...(contLayer > fi ? [cont] : [])];
            return targets;
        }
        if (parts.length === 2) {
            const sys = systems.find(s => s.id === parts[0]);
            const siblings = (sys?.containers || [])
                .filter(c => c.id !== parts[1])
                .map(c => ({ id: `${parts[0]}.${c.id}`, label: c.label || c.id }))
                .filter(t => layerIndex(resolveMetaLayer(t.id)) > fi);
            const dss = (sys?.datastores || [])
                .map(ds => ({ id: `${parts[0]}.${ds.id}`, label: ds.label || ds.id }))
                .filter(t => layerIndex(resolveMetaLayer(t.id)) > fi);
            return [...siblings, ...dss];
        }
        return [];
    };

    const submitReverseRelation = async (fromId: string, toId: string) => {
        await updateArchitecture((arch: ArchitectureJSON) => {
            const existing = (arch.architecture?.relations || []);
            const filtered = existing.filter(r => !(r.from === fromId && r.to === toId));
            const relations = [...filtered, { from: toId, to: fromId }];
            return { ...arch, architecture: { ...arch.architecture, relations } };
        });
    };

    const submitReplaceRelation = async (fromId: string, toId: string, newTargetId: string) => {
        await updateArchitecture((arch: ArchitectureJSON) => {
            const existing = (arch.architecture?.relations || []);
            const filtered = existing.filter(r => !(r.from === fromId && r.to === toId));
            const relations = [...filtered, { from: fromId, to: newTargetId }];
            return { ...arch, architecture: { ...arch.architecture, relations } };
        });
    };

    const submitAddRelation = async (fromId: string, toId: string) => {
        await updateArchitecture((arch: ArchitectureJSON) => {
            const relations = [...(arch.architecture?.relations || [])];
            relations.push({ from: fromId, to: toId });
            return { ...arch, architecture: { ...arch.architecture, relations } };
        });
    };

    return (
        <div className="guided-panel">
            <div className="guided-section">
                <div className="guided-status-row">
                    {l1Complete ? <CheckCircle size={16} color="#22c55e" /> : <AlertCircle size={16} color="#f59e0b" />}
                    <span className={`guided-status ${l1Complete ? 'ok' : 'warn'}`}>{l1Complete ? 'Complete' : 'Incomplete'}</span>
                </div>
                <div className="guided-header">
                    <span>L1: System Context</span>
                    <div className="guided-header-actions">
                        <Button variant="ghost" size="sm" onClick={() => setActiveTab('diagram')} title="View Diagram"><Layout size={14} /></Button>
                        <Button variant="ghost" size="sm" onClick={() => setActiveTab('code')} title="View Code"><FileCode size={14} /></Button>
                    </div>
                </div>
                <div className="guided-stats">
                    <span>{systems.length} system{systems.length !== 1 ? 's' : ''}</span>
                    <span>{persons.length} actor{persons.length !== 1 ? 's' : ''}</span>
                    <span>{relations.length} relation{relations.length !== 1 ? 's' : ''}</span>
                    {requirements.length > 0 && <span>{requirements.length} requirement{requirements.length !== 1 ? 's' : ''}</span>}
                    {(scenarios.length > 0 || flows.length > 0) && <span>{scenarios.length + flows.length} scenario/flow</span>}
                </div>
                {isEditMode() && (
                    <div className="guided-actions">
                        <Input label="System name" value={systemName} onChange={(e) => setSystemName(e.target.value)} placeholder="e.g., WebApp" />
                        <Button variant="primary" onClick={submitAddSystem} type="button">Add System</Button>
                        <Input label="Actor name" value={personName} onChange={(e) => setPersonName(e.target.value)} placeholder="e.g., User" />
                        <Button variant="secondary" onClick={submitAddPerson} type="button">Add Actor</Button>
                    </div>
                )}
                <div className="guided-next">
                    <Button variant="primary" disabled={systems.length === 0} onClick={() => systems[0] && drillDown(systems[0].id, 'system')} type="button">Continue to L2</Button>
                </div>
                <div className="guided-checklist">
                    <div className="check-item">
                        {persons.length > 0 ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span>At least one actor</span>
                    </div>
                    <div className="check-item">
                        {systems.length > 0 ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span>At least one system</span>
                    </div>
                    <div className="check-item">
                        {hasL1Relations ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span>Actor ↔ System relationships defined</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{systemsWithDescriptions > 0 ? `${systemsWithDescriptions}/${systems.length} systems documented` : 'Document system descriptions'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{l1ReqTagged > 0 ? `${l1ReqTagged} requirements tagged to actors/systems` : 'Tag requirements to actors/systems'}</span>
                    </div>
                </div>
                {layeringViolationDetails.length > 0 && (
                    <div className="guided-actions">
                        <label>Layering violations</label>
                        {(
                            showLayeringDetails ? layeringViolationDetails : layeringViolationDetails.slice(0, 5)
                        ).map(v => (
                            <div key={`${v.from}->${v.to}`} className="row">
                                <span className="text-link" onClick={() => {
                                    const parts = v.from.split('.');
                                    if (parts.length === 1) {
                                        drillDown(parts[0], 'system');
                                    } else if (parts.length === 2) {
                                        drillDown(parts[1], 'container', parts[0]);
                                    } else if (parts.length === 3) {
                                        drillDown(parts[1], 'container', parts[0]);
                                    }
                                }}>{v.from}</span>
                                <span>→</span>
                                <span className="text-link" onClick={() => {
                                    const parts = v.to.split('.');
                                    if (parts.length === 1) {
                                        drillDown(parts[0], 'system');
                                    } else if (parts.length === 2) {
                                        drillDown(parts[1], 'container', parts[0]);
                                    } else if (parts.length === 3) {
                                        drillDown(parts[1], 'container', parts[0]);
                                    }
                                }}>{v.to}</span>
                                <span className="hint">{(v.fromLayer || '')} → {(v.toLayer || '')}</span>
                                <Button variant="secondary" type="button" onClick={() => submitReverseRelation(v.from, v.to)}>Reverse relation</Button>
                            </div>
                        ))}
                        {(
                            showLayeringDetails ? layeringViolationDetails : layeringViolationDetails.slice(0, 5)
                        ).map(v => {
                            const key = `${v.from}->${v.to}`;
                            const targets = getDownwardTargets(v.from);
                            const sel = layeringFixTargetMap[key] || targets[0]?.id || '';
                            return (
                                <div key={`${key}-fix`} className="form-group">
                                    <div className="row">
                                        <select value={sel} onChange={(e) => setLayeringFixTargetMap(prev => ({ ...prev, [key]: e.target.value }))}>
                                            <option value="">Select valid downward target</option>
                                            {targets.map(t => <option key={t.id} value={t.id}>{t.label}</option>)}
                                        </select>
                                        <Button variant="secondary" type="button" disabled={!sel} onClick={() => submitAddRelation(v.from, sel)}>Create suggested relation</Button>
                                        <Button variant="secondary" type="button" disabled={!sel} onClick={() => submitReplaceRelation(v.from, v.to, sel)}>Replace relation</Button>
                                    </div>
                                </div>
                            );
                        })}
                        {layeringViolationDetails.length > 5 && (
                            <div className="row">
                                <span className="text-link" onClick={() => setShowLayeringDetails(s => !s)}>
                                    {showLayeringDetails ? 'Show less' : 'View all'}
                                </span>
                            </div>
                        )}
                    </div>
                )}
            </div>

            <div className="guided-section">
                <div className="guided-status-row">
                    {systems.length > 0 && systems.every(s => (s.containers?.length ?? 0) > 0) && containersMissingRelations.length === 0 ? (
                        <CheckCircle size={16} color="#22c55e" />
                    ) : (
                        <AlertCircle size={16} color="#f59e0b" />
                    )}
                    <span className={`guided-status ${systems.length > 0 && systems.every(s => (s.containers?.length ?? 0) > 0) && containersMissingRelations.length === 0 ? 'ok' : 'warn'}`}>
                        {systems.length > 0 && systems.every(s => (s.containers?.length ?? 0) > 0) && containersMissingRelations.length === 0 ? 'Complete' : 'Incomplete'}
                    </span>
                </div>
                <div className="guided-header">
                    <span>L2: Container View</span>
                    <div className="guided-header-actions">
                        <Button variant="ghost" size="sm" onClick={() => setActiveTab('diagram')} title="View Diagram"><Layout size={14} /></Button>
                        <Button variant="ghost" size="sm" onClick={() => setActiveTab('code')} title="View Code"><FileCode size={14} /></Button>
                    </div>
                </div>
                <div className="guided-stats">
                    {(() => {
                        const sys = focusedSystemId ? systems.find(s => s.id === focusedSystemId) : systems[0];
                        const count = sys?.containers?.length ?? 0;
                        return <span>{count} container{count !== 1 ? 's' : ''}{sys ? ` in ${sys.label ?? sys.id}` : ''}</span>;
                    })()}
                    {hasContainerRelations && <span>relations defined</span>}
                    {requirements.length > 0 && <span>{requirements.length} requirements</span>}
                    {(scenarios.length > 0 || flows.length > 0) && <span>{scenarios.length} scenario{scenarios.length !== 1 ? 's' : ''}, {flows.length} flow{flows.length !== 1 ? 's' : ''}</span>}
                    {adrs.length > 0 && <span>{adrs.length} ADR{adrs.length !== 1 ? 's' : ''}</span>}
                </div>
                {isEditMode() && (
                    <div className="guided-actions">
                        <div className="form-group">
                            <label>Parent system</label>
                            <select value={containerParentId} onChange={(e) => setContainerParentId(e.target.value)}>
                                <option value="">Select system</option>
                                {systems.map(s => (
                                    <option key={s.id} value={s.id}>{s.label ?? s.id}</option>
                                ))}
                            </select>
                        </div>
                        <Input label="Container name" value={containerName} onChange={(e) => setContainerName(e.target.value)} placeholder="e.g., API" />
                        <Input label="Technology" value={containerTech} onChange={(e) => setContainerTech(e.target.value)} placeholder="e.g., Node.js" />
                        <div className="form-group checkbox-row">
                            <input id="gb-container-external" type="checkbox" checked={containerExternal} onChange={(e) => setContainerExternal(e.target.checked)} />
                            <label htmlFor="gb-container-external">External</label>
                        </div>
                        <Button variant="primary" onClick={submitAddContainer} type="button">Add Container</Button>
                    </div>
                )}
                <div className="guided-next">
                    <Button
                        variant="primary"
                        disabled={(() => {
                            const sys = focusedSystemId ? systems.find(s => s.id === focusedSystemId) : systems[0];
                            return !(sys && (sys.containers?.length ?? 0) > 0);
                        })()}
                        onClick={() => {
                            const sys = focusedSystemId ? systems.find(s => s.id === focusedSystemId) : systems[0];
                            const firstContainer = sys?.containers?.[0];
                            if (sys && firstContainer) {
                                drillDown(firstContainer.id, 'container', sys.id);
                            }
                        }}
                        type="button"
                    >
                        Continue to L3
                    </Button>
                </div>
                <div className="guided-checklist">
                    <div className="check-item">
                        {systems.every(s => (s.containers?.length ?? 0) > 0) ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span>All systems have at least one container</span>
                    </div>
                    <div className="check-item">
                        {containersMissingRelations.length === 0 ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span title={containersMissingRelations.length > 0 ? `Missing: ${containersMissingRelations.slice(0, 5).map(c => c.label || c.id).join(', ')}` : undefined}>
                            All containers have relations ({containersWithRelations}/{allContainers.length})
                        </span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{containersWithTech > 0 ? `${containersWithTech}/${allContainers.length} containers have technology` : 'Add technology to containers'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{l2ReqTagged > 0 ? `${l2ReqTagged} requirements tagged to systems/containers` : 'Tag requirements to systems/containers'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{adrs.length > 0 ? `${adrs.length} ADR${adrs.length !== 1 ? 's' : ''}` : 'Add ADRs for key decisions'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{containersWithRelations > 0 ? `${containersWithRelations}/${allContainers.length} containers have relations` : 'Add relations to containers'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{sharedDatastores > 0 ? `${sharedDatastores} shared datastore${sharedDatastores !== 1 ? 's' : ''} detected` : 'No shared datastores'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{layeringViolations > 0 ? `${layeringViolations} layering violation${layeringViolations !== 1 ? 's' : ''}` : 'No layering violations detected'}</span>
                    </div>
                </div>
                {containersMissingRelations.length > 0 && (
                    <div className="guided-actions">
                        <label>Quick fixes: add relations to containers</label>
                        {(
                            showAllContainerFixes ? containersMissingRelations : containersMissingRelations.slice(0, 3)
                        ).map(c => {
                            const sysId = containerParentById.get(c.id);
                            const fromId = sysId ? `${sysId}.${c.id}` : c.id;
                            const targets = getContainerTargets(c.id);
                            const sel = containerTargetMap[c.id] || targets[0]?.id || '';
                            return (
                                <div key={c.id} className="form-group">
                                    <div className="row">
                                        <span className="text-link" onClick={() => sysId ? drillDown(c.id, 'container', sysId) : drillDown(c.id, 'container')}>{c.label || c.id}</span>
                                        <span className="hint">Focus</span>
                                    </div>
                                    <select value={sel} onChange={(e) => setContainerTargetMap(prev => ({ ...prev, [c.id]: e.target.value }))}>
                                        <option value="">Select target</option>
                                        {targets.map(t => <option key={t.id} value={t.id}>{t.label}</option>)}
                                    </select>
                                    <Button variant="secondary" type="button" disabled={!sel} onClick={() => submitAddRelation(fromId, sel)}>Add relation</Button>
                                </div>
                            );
                        })}
                        {containersMissingRelations.length > 3 && (
                            <div className="row">
                                <span className="text-link" onClick={() => setShowAllContainerFixes(s => !s)}>
                                    {showAllContainerFixes ? 'Show less' : 'View all'}
                                </span>
                            </div>
                        )}
                    </div>
                )}
            </div>

            <div className="guided-section">
                <div className="guided-status-row">
                    {allContainers.length > 0 && allContainers.every(c => (c.components?.length ?? 0) > 0) && componentsMissingRelations.length === 0 ? (
                        <CheckCircle size={16} color="#22c55e" />
                    ) : (
                        <AlertCircle size={16} color="#f59e0b" />
                    )}
                    <span className={`guided-status ${allContainers.length > 0 && allContainers.every(c => (c.components?.length ?? 0) > 0) && componentsMissingRelations.length === 0 ? 'ok' : 'warn'}`}>
                        {allContainers.length > 0 && allContainers.every(c => (c.components?.length ?? 0) > 0) && componentsMissingRelations.length === 0 ? 'Complete' : 'Incomplete'}
                    </span>
                </div>
                <div className="guided-header">
                    <span>L3: Component View</span>
                    <div className="guided-header-actions">
                        <Button variant="ghost" size="sm" onClick={() => setActiveTab('diagram')} title="View Diagram"><Layout size={14} /></Button>
                        <Button variant="ghost" size="sm" onClick={() => setActiveTab('code')} title="View Code"><FileCode size={14} /></Button>
                    </div>
                </div>
                <div className="guided-stats">
                    {(() => {
                        const container = focusedContainerId ? allContainers.find(c => c.id === focusedContainerId) : allContainers[0];
                        const count = container?.components?.length ?? 0;
                        return <span>{count} component{count !== 1 ? 's' : ''}{container ? ` in ${container.label ?? container.id}` : ''}</span>;
                    })()}
                    {hasComponentRelations && <span>relations defined</span>}
                    {requirements.length > 0 && <span>{requirements.length} requirements</span>}
                    {(scenarios.length > 0 || flows.length > 0) && <span>{scenarios.length} scenario{scenarios.length !== 1 ? 's' : ''}, {flows.length} flow{flows.length !== 1 ? 's' : ''}</span>}
                    {adrs.length > 0 && <span>{adrs.length} ADR{adrs.length !== 1 ? 's' : ''}</span>}
                </div>
                {isEditMode() && (
                    <div className="guided-actions">
                        <div className="form-group">
                            <label>Parent container</label>
                            <select value={componentParentId} onChange={(e) => setComponentParentId(e.target.value)}>
                                <option value="">Select container</option>
                                {allContainers.map(c => (
                                    <option key={c.id} value={c.id}>{c.label ?? c.id}</option>
                                ))}
                            </select>
                        </div>
                        <Input label="Component name" value={componentName} onChange={(e) => setComponentName(e.target.value)} placeholder="e.g., AuthService" />
                        <Button variant="primary" onClick={submitAddComponent} type="button">Add Component</Button>
                    </div>
                )}
                <div className="guided-checklist">
                    <div className="check-item">
                        {allContainers.every(c => (c.components?.length ?? 0) > 0) ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span>All containers have at least one component</span>
                    </div>
                    <div className="check-item">
                        {componentsMissingRelations.length === 0 ? <CheckCircle size={14} color="#22c55e" /> : <AlertCircle size={14} color="#f59e0b" />}
                        <span title={componentsMissingRelations.length > 0 ? `Missing: ${componentsMissingRelations.slice(0, 5).map(c => c.label || c.id).join(', ')}` : undefined}>
                            All components have relations ({componentsWithRelations}/{componentsAll.length})
                        </span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{componentsWithTech > 0 ? `${componentsWithTech}/${componentsAll.length} components have technology` : 'Add technology to components (optional)'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{l3ReqTagged > 0 ? `${l3ReqTagged} requirements tagged to components/containers` : 'Tag requirements to components/containers'}</span>
                    </div>
                    <div className="check-item optional">
                        <Info size={14} />
                        <span>{componentsWithRelations > 0 ? `${componentsWithRelations}/${componentsAll.length} components have relations` : 'Add relations to components'}</span>
                    </div>
                </div>
                {componentsMissingRelations.length > 0 && (
                    <div className="guided-actions">
                        <label>Quick fixes: add relations to components</label>
                        {(
                            showAllComponentFixes ? componentsMissingRelations : componentsMissingRelations.slice(0, 3)
                        ).map(co => {
                            const parent = componentParentById.get(co.id);
                            const fromId = parent ? `${parent.sysId}.${parent.containerId}.${co.id}` : co.id;
                            const targets = getComponentTargets(co.id);
                            const sel = componentTargetMap[co.id] || targets[0]?.id || '';
                            return (
                                <div key={co.id} className="form-group">
                                    <div className="row">
                                        <span className="text-link" onClick={() => parent ? drillDown(parent.containerId, 'container', parent.sysId) : undefined}>{co.label || co.id}</span>
                                        <span className="hint">Focus</span>
                                    </div>
                                    <select value={sel} onChange={(e) => setComponentTargetMap(prev => ({ ...prev, [co.id]: e.target.value }))}>
                                        <option value="">Select target</option>
                                        {targets.map(t => <option key={t.id} value={t.id}>{t.label}</option>)}
                                    </select>
                                    <Button variant="secondary" type="button" disabled={!sel} onClick={() => submitAddRelation(fromId, sel)}>Add relation</Button>
                                </div>
                            );
                        })}
                        {componentsMissingRelations.length > 3 && (
                            <div className="row">
                                <span className="text-link" onClick={() => setShowAllComponentFixes(s => !s)}>
                                    {showAllComponentFixes ? 'Show less' : 'View all'}
                                </span>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
