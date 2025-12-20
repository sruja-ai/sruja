// apps/website/src/features/playground/components/DiagramPreview.tsx
// Minimal React Flow viewer for displaying architecture diagrams in code blocks
import { useCallback, useMemo, useState, useRef, useEffect } from 'react';
import type { ArchitectureJSON } from '@sruja/diagram';
import {
    ReactFlow,
    ReactFlowProvider,
    useNodesState,
    useEdgesState,
    Controls,
    Background,
    BackgroundVariant,
    type Node,
    type Edge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { jsonToReactFlow, nodeTypes, edgeTypes, applySrujaLayout, Legend, exportSVG, exportPNGFromSVG } from '@sruja/diagram';
import type { C4NodeData } from '@sruja/diagram';

interface DiagramPreviewProps {
    data: ArchitectureJSON;
}

async function transformToFlow(
    data: ArchitectureJSON,
    level: 'L0' | 'L1' | 'L2' | 'L3',
    focusedSystemId?: string,
    focusedContainerId?: string
): Promise<{ nodes: Node<C4NodeData>[]; edges: Edge[] }> {
    const result = jsonToReactFlow(data, {
        level,
        focusedSystemId,
        focusedContainerId,
        expandedNodes: new Set<string>(),
    });
    const laidOut = await applySrujaLayout(result.nodes, result.edges, data, {
        level,
        focusedSystemId,
        focusedContainerId,
        direction: 'LR',
        expandedNodes: new Set<string>(),
    });
    return { nodes: laidOut.nodes, edges: laidOut.edges };
}

function FlowContent({ data, level, systemId, containerId, onStatsChange }: { data: ArchitectureJSON; level: 'L0'|'L1'|'L2'|'L3'; systemId?: string; containerId?: string; onStatsChange?: (nodes: number, edges: number) => void }) {
    const [initialNodes, setInitialNodes] = useState<Node<C4NodeData>[]>([]);
    const [initialEdges, setInitialEdges] = useState<Edge[]>([]);
    
    useEffect(() => {
        transformToFlow(data, level, systemId, containerId).then(({ nodes, edges }) => {
            setInitialNodes(nodes);
            setInitialEdges(edges);
        });
    }, [data, level, systemId, containerId]);
    const [nodes, , onNodesChange] = useNodesState<Node<C4NodeData>>(initialNodes);
    const [edges, , onEdgesChange] = useEdgesState<Edge>(initialEdges);
    const rfRef = useRef<any>(null);
    const onInit = useCallback((instance: any) => {
        rfRef.current = instance;
        setTimeout(() => instance.fitView({ padding: 0.2 }), 100);
    }, []);

    const stats = useMemo(() => ({ n: nodes.length, e: edges.length }), [nodes, edges]);
    useEffect(() => { if (onStatsChange) onStatsChange(stats.n, stats.e); }, [stats, onStatsChange]);

    return (
        <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onInit={onInit}
            fitView
            attributionPosition="bottom-left"
        >
            <Controls />
            <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
            <div style={{ position: 'absolute', right: 8, bottom: 8, zIndex: 10 }}>
                <button
                    onClick={() => { try { rfRef.current?.fitView({ padding: 0.2 }); } catch {} }}
                    style={{ padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-primary)', fontSize: 12, cursor: 'pointer' }}
                >
                    Fit View
                </button>
                <button
                    onClick={() => {
                        try {
                            const svg = exportSVG(nodes as any, edges as any)
                            const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' })
                            const url = URL.createObjectURL(blob)
                            const a = document.createElement('a')
                            a.href = url
                            a.download = 'diagram.svg'
                            document.body.appendChild(a)
                            a.click()
                            document.body.removeChild(a)
                            URL.revokeObjectURL(url)
                        } catch {}
                    }}
                    style={{ marginLeft: 8, padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-primary)', fontSize: 12, cursor: 'pointer' }}
                >
                    Export SVG
                </button>
                <button
                    onClick={async () => {
                        try {
                            const svg = exportSVG(nodes as any, edges as any)
                            const png = await exportPNGFromSVG(svg, 2)
                            const a = document.createElement('a')
                            a.href = png
                            a.download = 'diagram.png'
                            document.body.appendChild(a)
                            a.click()
                            document.body.removeChild(a)
                        } catch {}
                    }}
                    style={{ marginLeft: 8, padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-primary)', fontSize: 12, cursor: 'pointer' }}
                >
                    Export PNG
                </button>
            </div>
            <div style={{ position: 'absolute', left: 8, bottom: 8, zIndex: 9 }}>
                <Legend />
            </div>
        </ReactFlow>
    );
}

export function DiagramPreview({ data }: DiagramPreviewProps) {
    const [level, setLevel] = useState<'L0'|'L1'|'L2'|'L3'>('L1');
    const systems = (data?.architecture?.systems || []).map(s => ({ id: s.id, label: s.label || s.id }));
    const [systemId, setSystemId] = useState<string | undefined>(systems[0]?.id);
    const containers = (data?.architecture?.systems || []).find(s => s.id === systemId)?.containers || [];
    const [containerId, setContainerId] = useState<string | undefined>(containers[0]?.id);

    const showSystem = level === 'L2' || level === 'L3';
    const showContainer = level === 'L3';

    const onMountLoadState = useCallback(() => {
        try {
            const hash = typeof window !== 'undefined' ? window.location.hash : '';
            const qs = new URLSearchParams((hash || '').replace(/^#/, ''));
            const lv = qs.get('level') as ('L0'|'L1'|'L2'|'L3') | null;
            const sys = qs.get('system');
            const con = qs.get('container');
            if (lv) setLevel(lv);
            if (sys) setSystemId(sys);
            if (con) setContainerId(con);
        } catch {}
    }, []);
    const writeState = useCallback((lv: string, sys?: string, con?: string) => {
        try {
            const qs = new URLSearchParams();
            qs.set('level', lv);
            if (sys) qs.set('system', sys);
            if (con) qs.set('container', con);
            const nextHash = `#${qs.toString()}`;
            if (typeof window !== 'undefined') {
                window.history.replaceState(null, '', `${window.location.pathname}${window.location.search}${nextHash}`);
            }
        } catch {}
    }, []);

    const [counts, setCounts] = useState<{ nodes: number; edges: number }>({ nodes: 0, edges: 0 });
    useEffect(() => { onMountLoadState(); }, [onMountLoadState]);

    return (
        <div style={{ width: '100%', height: '100%', minHeight: 400, position: 'relative' }}>
            <div style={{ position: 'absolute', top: 8, right: 8, zIndex: 20, display: 'flex', gap: 8, alignItems: 'center' }}>
                <select value={level} onChange={(e) => {
                    const v = e.target.value as 'L0'|'L1'|'L2'|'L3';
                    setLevel(v);
                    writeState(v, systemId, containerId);
                }} style={{ padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-primary)', fontSize: 12 }}>
                    <option value="L0">Landscape</option>
                    <option value="L1">System Context</option>
                    <option value="L2">Container</option>
                    <option value="L3">Component</option>
                </select>
                {showSystem && (
                    <select value={systemId || ''} onChange={(e) => {
                        const v = e.target.value || undefined;
                        setSystemId(v);
                        const nextContainers = (data?.architecture?.systems || []).find(s => s.id === v)?.containers || [];
                        setContainerId(nextContainers[0]?.id);
                        writeState(level, v, nextContainers[0]?.id);
                    }} style={{ padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-primary)', fontSize: 12 }}>
                        {systems.map(s => (<option key={s.id} value={s.id}>{s.label}</option>))}
                    </select>
                )}
                {showContainer && (
                    <select value={containerId || ''} onChange={(e) => { const v = e.target.value || undefined; setContainerId(v); writeState(level, systemId, v); }} style={{ padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-primary)', fontSize: 12 }}>
                        {(((data?.architecture?.systems || []).find(s => s.id === systemId)?.containers) || []).map(c => (<option key={c.id} value={c.id}>{c.label || c.id}</option>))}
                    </select>
                )}
            </div>
            <div style={{ position: 'absolute', top: 8, left: 8, zIndex: 20, padding: '6px 10px', borderRadius: 6, border: '1px solid var(--color-border)', background: 'var(--color-background)', color: 'var(--color-text-secondary)', fontSize: 12 }}>
                {counts.nodes} nodes · {counts.edges} edges
            </div>
            <ReactFlowProvider>
                <FlowContent data={data} level={level} systemId={systemId} containerId={containerId} onStatsChange={(n,e) => setCounts({ nodes: n, edges: e })} />
            </ReactFlowProvider>
        </div>
    );
}
