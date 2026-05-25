import * as vscode from "vscode";
import { getSrujaLspPath } from "./config";
import { runCli } from "./cliRunner";

let explorerPanel: vscode.WebviewPanel | undefined;

function getNonce(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let out = "";
  for (let i = 0; i < 32; i++) {
    out += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return out;
}

function getLspPath(): string {
  return getSrujaLspPath(
    vscode.workspace.getConfiguration("sruja").get<string>("lsp.path")
  );
}

function getWorkspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

export function registerExplorerCommands(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.openArchitectureExplorer", async () => {
      const root = getWorkspaceRoot();
      if (!root) {
        vscode.window.showWarningMessage("Open a workspace folder to use the Architecture Explorer.");
        return;
      }

      if (explorerPanel) {
        explorerPanel.reveal(vscode.ViewColumn.Beside);
        await refreshExplorer(context, root);
        return;
      }

      explorerPanel = vscode.window.createWebviewPanel(
        "srujaArchitectureExplorer",
        "Sruja – Architecture Explorer",
        vscode.ViewColumn.Beside,
        {
          enableScripts: true,
          retainContextWhenHidden: true,
          localResourceRoots: [
            vscode.Uri.joinPath(context.extensionUri, "media"),
          ],
        }
      );

      explorerPanel.webview.html = getLoadingHtml();

      explorerPanel.webview.onDidReceiveMessage(
        async (msg) => {
          if (msg.type === "goToDefinition" && msg.nodeId) {
            await goToDefinition(context, msg.nodeId);
          }
          if (msg.type === "refresh") {
            await refreshExplorer(context, root);
          }
        },
        undefined,
        context.subscriptions
      );

      explorerPanel.onDidDispose(() => {
        explorerPanel = undefined;
      });

      await refreshExplorer(context, root);
    })
  );
}

async function refreshExplorer(context: vscode.ExtensionContext, root: string) {
  if (!explorerPanel) { return; }

  explorerPanel.webview.html = getLoadingHtml();

  try {
    const srujaPath = getLspPath();
    const result = await runCli(srujaPath, ["explore", "-r", root], root);

    if (result.code !== 0) {
      explorerPanel.webview.html = getErrorHtml(
        `CLI exited with code ${result.code}.\n${result.stderr}`
      );
      return;
    }

    const model = JSON.parse(result.stdout);
    const d3Uri = explorerPanel.webview.asWebviewUri(
      vscode.Uri.joinPath(context.extensionUri, "media", "d3.v7.min.js")
    );
    explorerPanel.webview.html = getExplorerHtml(explorerPanel.webview, model, d3Uri.toString());
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    if (explorerPanel) {
      explorerPanel.webview.html = getErrorHtml(msg);
    }
  }
}

async function goToDefinition(
  context: vscode.ExtensionContext,
  nodeId: string
) {
  const srujaFiles = await vscode.workspace.findFiles("**/*.sruja", "**/node_modules/**", 20);
  for (const uri of srujaFiles) {
    const doc = await vscode.workspace.openTextDocument(uri);
    const text = doc.getText();
    const pattern = new RegExp(`\\b${escapeRegex(nodeId)}\\s*=\\s*`);
    const match = pattern.exec(text);
    if (match) {
      const pos = doc.positionAt(match.index);
      const editor = await vscode.window.showTextDocument(doc, vscode.ViewColumn.One);
      editor.selection = new vscode.Selection(pos, pos);
      editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
      return;
    }
  }
  vscode.window.showInformationMessage(`Could not find definition for "${nodeId}" in .sruja files.`);
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function getLoadingHtml(): string {
  return `<!DOCTYPE html><html><head>
<style>body{display:flex;align-items:center;justify-content:center;height:100vh;margin:0;
background:var(--vscode-editor-background);color:var(--vscode-editor-foreground);
font-family:var(--vscode-font-family);}
.spinner{width:40px;height:40px;border:3px solid var(--vscode-editor-foreground);
border-top-color:transparent;border-radius:50%;animation:spin .8s linear infinite;}
@keyframes spin{to{transform:rotate(360deg)}}</style></head>
<body><div style="text-align:center"><div class="spinner" style="margin:0 auto 16px"></div>
<div>Scanning architecture…</div></div></body></html>`;
}

function getErrorHtml(msg: string): string {
  const escaped = msg.replace(/&/g,"&amp;").replace(/</g,"&lt;");
  return `<!DOCTYPE html><html><head>
<style>body{padding:24px;background:var(--vscode-editor-background);
color:var(--vscode-errorForeground,#f44);font-family:var(--vscode-font-family);}
pre{white-space:pre-wrap;}</style></head>
<body><h3>Architecture Explorer Error</h3><pre>${escaped}</pre></body></html>`;
}

function getExplorerHtml(webview: vscode.Webview, model: unknown, d3Src: string): string {
  const nonce = getNonce();
  const json = JSON.stringify(model).replace(/</g, "\\u003c");
  const csp = `default-src 'none'; img-src ${webview.cspSource} data:; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}' ${webview.cspSource};`;
  return `<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy"
  content="${csp}">
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{overflow:hidden;width:100vw;height:100vh;
  background:var(--vscode-editor-background);color:var(--vscode-editor-foreground);
  font-family:var(--vscode-font-family);font-size:12px}

/* --- toolbar --- */
#toolbar{position:fixed;top:0;left:0;right:0;height:36px;
  display:flex;align-items:center;gap:4px;padding:0 8px;z-index:100;
  background:var(--vscode-sideBar-background,#252526);
  border-bottom:1px solid var(--vscode-panel-border,#3c3c3c)}
#toolbar button{background:transparent;color:var(--vscode-button-foreground,#ccc);
  border:1px solid var(--vscode-button-border,#555);border-radius:3px;
  padding:2px 8px;cursor:pointer;font-size:11px;font-family:inherit}
#toolbar button:hover{background:var(--vscode-button-hoverBackground,#3a3a3a)}
#toolbar button.active{background:var(--vscode-button-background,#0e639c);
  border-color:var(--vscode-focusBorder,#007fd4)}
#toolbar .sep{width:1px;height:18px;background:var(--vscode-panel-border,#3c3c3c)}
#breadcrumb{margin-left:8px;font-size:11px;color:var(--vscode-descriptionForeground,#888)}
#breadcrumb span{cursor:pointer;text-decoration:underline}
#search{margin-left:auto;background:var(--vscode-input-background,#3c3c3c);
  color:var(--vscode-input-foreground,#ccc);border:1px solid var(--vscode-input-border,#555);
  border-radius:3px;padding:2px 6px;font-size:11px;width:150px;font-family:inherit}
#refreshBtn{margin-left:4px}

/* --- canvas --- */
#canvas{position:fixed;top:36px;left:0;right:0;bottom:24px}
svg{width:100%;height:100%}

/* --- tooltip --- */
#tooltip{position:fixed;pointer-events:none;background:var(--vscode-editorHoverWidget-background,#2d2d2d);
  color:var(--vscode-editorHoverWidget-foreground,#ccc);
  border:1px solid var(--vscode-editorHoverWidget-border,#454545);
  border-radius:4px;padding:8px 10px;font-size:11px;max-width:320px;
  box-shadow:0 4px 12px rgba(0,0,0,.4);z-index:200;display:none;line-height:1.5}
#tooltip .tt-title{font-weight:600;font-size:12px;margin-bottom:4px}
#tooltip .tt-kind{opacity:.6;font-size:10px;text-transform:uppercase;letter-spacing:.5px}
#tooltip .tt-metric{margin-top:4px}
#tooltip .tt-metric b{color:var(--vscode-textLink-foreground,#3794ff)}

/* --- summary bar --- */
#summary{position:fixed;bottom:0;left:0;right:0;height:24px;
  display:flex;align-items:center;gap:12px;padding:0 10px;font-size:11px;
  background:var(--vscode-statusBar-background,#007acc);
  color:var(--vscode-statusBar-foreground,#fff)}
#summary .val{font-weight:600}

/* --- node styles --- */
.node-circle{stroke-width:1.5;cursor:pointer;transition:opacity .15s}
.node-circle:hover{opacity:.8}
.node-label{font-size:10px;pointer-events:none;fill:var(--vscode-editor-foreground);text-anchor:middle}
.edge-line{stroke:var(--vscode-editorLineNumber-foreground,#666);stroke-width:1;fill:none}
.edge-arrow{fill:var(--vscode-editorLineNumber-foreground,#666)}
.community-hull{fill-opacity:.06;stroke-width:1.5;stroke-dasharray:4,3}
</style>
</head>
<body>

<div id="toolbar">
  <button data-overlay="structure" class="active">Structure</button>
  <button data-overlay="centrality">Centrality</button>
  <button data-overlay="coupling">Coupling</button>
  <button data-overlay="drift">Drift</button>
  <button data-overlay="communities">Communities</button>
  <button data-overlay="cycles">Cycles</button>
  <div class="sep"></div>
  <div id="breadcrumb"></div>
  <input id="search" type="text" placeholder="Search nodes…">
  <button id="refreshBtn" title="Refresh">&#x21bb;</button>
</div>

<div id="canvas"></div>
<div id="tooltip"></div>
<div id="summary"></div>

<script nonce="${nonce}" src="${d3Src}"><\/script>
<script nonce="${nonce}">
(function(){
const vscode = acquireVsCodeApi();
const MODEL = ${json};

// ---- color palettes ----
const KIND_COLORS = {
  system:'#4fc3f7',container:'#81c784',component:'#ffb74d',
  database:'#ce93d8',queue:'#f06292',external_api:'#90a4ae',
  frontend:'#aed581',service:'#4dd0e1',module:'#78909c',person:'#ffd54f'
};
const COUPLING_COLORS = {main_sequence:'#81c784',zone_of_pain:'#ef5350',zone_of_uselessness:'#ffa726',unknown:'#78909c'};
const HEALTH_COLORS = {healthy:'#81c784',minordrift:'#ffa726',minor_drift:'#ffa726',
  significantdrift:'#ef5350',significant_drift:'#ef5350',criticaldrift:'#c62828',critical_drift:'#c62828'};
const COMMUNITY_PALETTE = ['#42a5f5','#66bb6a','#ffa726','#ab47bc','#ef5350','#26c6da','#8d6e63','#78909c','#d4e157','#ec407a'];

const width = () => document.getElementById('canvas').clientWidth;
const height = () => document.getElementById('canvas').clientHeight;

let currentOverlay = 'structure';
let drillStack = []; // [{level, focusId}]
let simulation, svg, g, nodeG, edgeG, hullG, tooltip;

function edgeKey(e) {
  const s = e.source && e.source.id ? e.source.id : e.source;
  const t = e.target && e.target.id ? e.target.id : e.target;
  return String(s) + '->' + String(t);
}

// ---- init ----
function init() {
  svg = d3.select('#canvas').append('svg');
  const defs = svg.append('defs');
  defs.append('marker').attr('id','arrowhead').attr('viewBox','0 -5 10 10')
    .attr('refX',20).attr('refY',0).attr('markerWidth',6).attr('markerHeight',6)
    .attr('orient','auto').append('path').attr('d','M0,-4L10,0L0,4').attr('class','edge-arrow');
  g = svg.append('g');
  hullG = g.append('g').attr('class','hulls');
  edgeG = g.append('g').attr('class','edges');
  nodeG = g.append('g').attr('class','nodes');
  tooltip = d3.select('#tooltip');

  svg.call(d3.zoom().scaleExtent([.1,8]).on('zoom', e => g.attr('transform', e.transform)));

  renderLevel(null);
  renderSummary();
  bindToolbar();
}

// ---- data filtering by drill level ----
function nodesForLevel(focusId) {
  if (!focusId) {
    return MODEL.nodes.filter(n => !n.parent_id);
  }
  return MODEL.nodes.filter(n => n.parent_id === focusId);
}

function edgesForNodes(nodeSet) {
  const ids = new Set(nodeSet.map(n => n.id));
  return MODEL.edges.filter(e => ids.has(e.source) && ids.has(e.target));
}

// ---- main render ----
function renderLevel(focusId) {
  const nodes = nodesForLevel(focusId);
  const edges = edgesForNodes(nodes);

  if (nodes.length === 0 && focusId) {
    vscode.postMessage({type:'goToDefinition', nodeId: focusId});
    return;
  }

  const w = width(), h = height();

  if (simulation) simulation.stop();

  // community centroids for cluster force
  const comCentroids = {};
  if (MODEL.communities) {
    const ids = new Set(nodes.map(n => n.id));
    MODEL.communities.forEach((c,i) => {
      const members = c.member_ids.filter(m => ids.has(m));
      if (members.length > 0) {
        const angle = (2 * Math.PI * i) / Math.max(MODEL.communities.length, 1);
        comCentroids[c.id] = {x: w/2 + Math.cos(angle)*w*0.25, y: h/2 + Math.sin(angle)*h*0.25, members: new Set(members)};
      }
    });
  }

  const simNodes = nodes.map(n => ({...n, x: w/2 + (Math.random()-.5)*w*.4, y: h/2 + (Math.random()-.5)*h*.4}));
  const simEdges = edges.map(e => ({...e, source: e.source, target: e.target}));

  simulation = d3.forceSimulation(simNodes)
    .force('link', d3.forceLink(simEdges).id(d => d.id).distance(100).strength(.3))
    .force('charge', d3.forceManyBody().strength(-300))
    .force('center', d3.forceCenter(w/2, h/2))
    .force('collide', d3.forceCollide().radius(d => nodeRadius(d)+8))
    .force('cluster', clusterForce(comCentroids, .03));

  // edges
  edgeG.selectAll('*').remove();
  const edgeSel = edgeG.selectAll('line').data(simEdges, edgeKey);
  const edgeEnter = edgeSel.enter().append('line').attr('class','edge-line').attr('marker-end','url(#arrowhead)');

  // community hulls
  hullG.selectAll('*').remove();

  // nodes
  nodeG.selectAll('*').remove();
  const nodeEnter = nodeG.selectAll('g').data(simNodes, d => d.id).enter().append('g');
  nodeEnter.append('circle').attr('class','node-circle')
    .attr('r', d => nodeRadius(d))
    .call(drag(simulation));
  nodeEnter.append('text').attr('class','node-label').attr('dy', d => nodeRadius(d)+12)
    .text(d => d.label.length > 18 ? d.label.slice(0,16)+'…' : d.label);

  nodeEnter.on('click', (ev, d) => {
    if (d.children_count > 0) {
      drillStack.push(focusId);
      renderLevel(d.id);
      updateBreadcrumb(d.id);
    } else {
      vscode.postMessage({type:'goToDefinition', nodeId: d.id});
    }
  }).on('contextmenu', (ev, d) => {
    ev.preventDefault();
    vscode.postMessage({type:'goToDefinition', nodeId: d.id});
  }).on('mouseover', (ev, d) => showTooltip(ev, d))
    .on('mouseout', () => tooltip.style('display','none'));

  simulation.on('tick', () => {
    edgeEnter.attr('x1',d=>d.source.x).attr('y1',d=>d.source.y)
      .attr('x2',d=>d.target.x).attr('y2',d=>d.target.y);
    nodeEnter.attr('transform', d => 'translate('+d.x+','+d.y+')');
    renderHulls(simNodes, comCentroids);
  });

  applyOverlay(currentOverlay, simNodes, simEdges);
}

// ---- cluster force ----
function clusterForce(centroids, strength) {
  let nodes;
  function force(alpha) {
    for (const n of nodes) {
      const cid = n.metrics?.community_id;
      if (cid != null && centroids[cid]) {
        const c = centroids[cid];
        n.vx += (c.x - n.x) * alpha * strength;
        n.vy += (c.y - n.y) * alpha * strength;
      }
    }
  }
  force.initialize = (n) => { nodes = n; };
  return force;
}

// ---- community hulls ----
function renderHulls(simNodes, centroids) {
  if (currentOverlay !== 'communities') { hullG.selectAll('*').remove(); return; }
  hullG.selectAll('*').remove();
  Object.entries(centroids).forEach(([cid, info], i) => {
    const pts = simNodes.filter(n => info.members.has(n.id)).map(n => [n.x, n.y]);
    if (pts.length < 3) return;
    const hull = d3.polygonHull(pts);
    if (!hull) return;
    const color = COMMUNITY_PALETTE[i % COMMUNITY_PALETTE.length];
    hullG.append('path')
      .attr('d', 'M'+hull.map(p=>p.join(',')).join('L')+'Z')
      .attr('class','community-hull')
      .attr('fill', color).attr('stroke', color);
  });
}

// ---- overlays ----
function applyOverlay(name, simNodes, simEdges) {
  const circles = nodeG.selectAll('circle');
  const lines = edgeG.selectAll('line');

  circles.attr('r', d => nodeRadius(d));

  switch(name) {
    case 'structure':
      circles.attr('fill', d => KIND_COLORS[d.kind]||'#78909c')
        .attr('stroke', d => d3.color(KIND_COLORS[d.kind]||'#78909c').darker(.5))
        .attr('stroke-dasharray', null).attr('stroke-width', 1.5);
      lines.attr('stroke', null).attr('stroke-dasharray', null).attr('stroke-width', 1).attr('class','edge-line');
      break;

    case 'centrality':
      const maxC = d3.max(simNodes, d => d.metrics.centrality) || 1;
      circles.attr('r', d => 8 + (d.metrics.centrality / maxC) * 24)
        .attr('fill', d => KIND_COLORS[d.kind]||'#78909c')
        .attr('stroke', d => d.metrics.is_hotspot ? '#ff9800' : d3.color(KIND_COLORS[d.kind]||'#78909c').darker(.5))
        .attr('stroke-width', d => d.metrics.is_hotspot ? 3 : 1.5)
        .attr('stroke-dasharray', null);
      lines.attr('stroke', null).attr('stroke-dasharray', null).attr('stroke-width', 1).attr('class','edge-line');
      break;

    case 'coupling':
      circles.attr('fill', d => COUPLING_COLORS[d.metrics.coupling_zone]||'#78909c')
        .attr('stroke', d => d3.color(COUPLING_COLORS[d.metrics.coupling_zone]||'#78909c').darker(.6))
        .attr('stroke-width', d => 1.5 + d.metrics.instability * 4)
        .attr('stroke-dasharray', null);
      lines.attr('stroke', null).attr('stroke-dasharray', null).attr('stroke-width', 1).attr('class','edge-line');
      break;

    case 'drift':
      circles.attr('fill', d => {
          const h = d.metrics.health.replace(/_/g,'');
          return HEALTH_COLORS[h] || KIND_COLORS[d.kind] || '#78909c';
        })
        .attr('stroke', d => {
          const h = d.metrics.health.replace(/_/g,'');
          return d3.color(HEALTH_COLORS[h] || '#78909c').darker(.5);
        })
        .attr('stroke-width', d => d.metrics.drift_count > 0 ? 3 : 1.5)
        .attr('stroke-dasharray', null);
      lines.attr('stroke', d => d.has_drift ? '#ef5350' : null)
        .attr('stroke-dasharray', d => d.has_drift ? '6,3' : null)
        .attr('stroke-width', d => d.has_drift ? 2 : 1);
      break;

    case 'communities':
      circles.attr('fill', d => {
          const cid = d.metrics.community_id;
          return cid != null ? COMMUNITY_PALETTE[(cid-1)%COMMUNITY_PALETTE.length] : '#78909c';
        })
        .attr('stroke', d => {
          const cid = d.metrics.community_id;
          return cid != null ? d3.color(COMMUNITY_PALETTE[(cid-1)%COMMUNITY_PALETTE.length]).darker(.5) : '#555';
        })
        .attr('stroke-width', 1.5).attr('stroke-dasharray', null);
      lines.attr('stroke', d => {
          const sn = simNodes.find(n => (n.id||'') === (d.source.id||d.source));
          const tn = simNodes.find(n => (n.id||'') === (d.target.id||d.target));
          if (sn && tn && sn.metrics.community_id === tn.metrics.community_id) return null;
          return 'rgba(255,255,255,0.15)';
        }).attr('stroke-dasharray', null).attr('stroke-width', 1);
      break;

    case 'cycles':
      const cycleNodes = new Set(MODEL.cycles.flatMap(c => c.nodes));
      const cycleEdges = new Set();
      MODEL.cycles.forEach(c => {
        const s = new Set(c.nodes);
        MODEL.edges.forEach(e => { if (s.has(e.source) && s.has(e.target)) cycleEdges.add(e.source+'->'+e.target); });
      });
      circles.attr('fill', d => cycleNodes.has(d.id) ? '#ef5350' : (KIND_COLORS[d.kind]||'#78909c'))
        .attr('stroke', d => cycleNodes.has(d.id) ? '#b71c1c' : d3.color(KIND_COLORS[d.kind]||'#78909c').darker(.5))
        .attr('stroke-width', d => cycleNodes.has(d.id) ? 3 : 1.5)
        .attr('stroke-dasharray', null);
      lines.attr('stroke', d => {
          const key = (d.source.id||d.source)+'->'+(d.target.id||d.target);
          return cycleEdges.has(key) ? '#ef5350' : null;
        })
        .attr('stroke-width', d => {
          const key = (d.source.id||d.source)+'->'+(d.target.id||d.target);
          return cycleEdges.has(key) ? 2.5 : 1;
        })
        .attr('stroke-dasharray', d => {
          const key = (d.source.id||d.source)+'->'+(d.target.id||d.target);
          return cycleEdges.has(key) ? '8,4' : null;
        });
      break;
  }
}

function nodeRadius(d) {
  if (d.children_count > 0) return 16;
  return 10;
}

// ---- tooltip ----
function showTooltip(ev, d) {
  let metricHtml = '';
  switch(currentOverlay) {
    case 'centrality':
      metricHtml = '<div class="tt-metric"><b>Centrality:</b> '+(d.metrics.centrality*100).toFixed(1)+'%'
        +(d.metrics.is_hotspot?' (hotspot)':'')+'</div>'; break;
    case 'coupling':
      metricHtml = '<div class="tt-metric"><b>Instability:</b> '+d.metrics.instability.toFixed(2)
        +' &middot; <b>Zone:</b> '+d.metrics.coupling_zone.replace(/_/g,' ')+'</div>'; break;
    case 'drift':
      metricHtml = '<div class="tt-metric"><b>Drifts:</b> '+d.metrics.drift_count
        +(d.metrics.drift_severity_max?' ('+d.metrics.drift_severity_max+')':'')
        +' &middot; <b>Health:</b> '+d.metrics.health.replace(/_/g,' ')+'</div>'; break;
    case 'communities':
      metricHtml = '<div class="tt-metric"><b>Community:</b> '+(d.metrics.community_id??'none')+'</div>'; break;
    case 'cycles':
      metricHtml = '<div class="tt-metric">'+(d.metrics.is_in_cycle?'<b style="color:#ef5350">In cycle</b>':'Not in cycle')+'</div>'; break;
    default:
      metricHtml = d.technology ? '<div class="tt-metric"><b>Tech:</b> '+d.technology+'</div>' : '';
  }

  tooltip.html(
    '<div class="tt-kind">'+d.kind+'</div>'
    +'<div class="tt-title">'+d.label+'</div>'
    +(d.description?'<div style="margin-top:4px;opacity:.8">'+d.description+'</div>':'')
    +metricHtml
    +(d.children_count>0?'<div style="margin-top:4px;opacity:.6">Click to drill down ('+d.children_count+' children)</div>':'')
  ).style('display','block')
   .style('left', (ev.pageX+12)+'px')
   .style('top', (ev.pageY-10)+'px');
}

// ---- breadcrumb ----
function updateBreadcrumb(focusId) {
  const bc = document.getElementById('breadcrumb');
  const parts = [];
  let id = focusId;
  while (id) {
    const n = MODEL.nodes.find(n => n.id === id);
    parts.unshift({id, label: n?n.label:id});
    id = n ? n.parent_id : null;
  }
  parts.unshift({id: null, label: 'Root'});
  bc.textContent = '';
  for (let i = 0; i < parts.length; i++) {
    const p = parts[i];
    if (i < parts.length - 1) {
      const s = document.createElement('span');
      s.textContent = p.label;
      s.dataset.id = p.id || '';
      s.addEventListener('click', () => drillTo(s.dataset.id));
      bc.appendChild(s);
      const sep = document.createTextNode(' / ');
      bc.appendChild(sep);
    } else {
      const b = document.createElement('b');
      b.textContent = p.label;
      bc.appendChild(b);
    }
  }
}

function drillTo(id) {
  drillStack = [];
  renderLevel(id || null);
  if (!id) {
    document.getElementById('breadcrumb').innerHTML = '';
  } else {
    updateBreadcrumb(id);
  }
}

// ---- toolbar ----
function bindToolbar() {
  document.querySelectorAll('#toolbar button[data-overlay]').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('#toolbar button[data-overlay]').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      currentOverlay = btn.dataset.overlay;
      const simNodes = nodeG.selectAll('g').data();
      const simEdges = edgeG.selectAll('line').data();
      applyOverlay(currentOverlay, simNodes, simEdges);
      if (currentOverlay === 'communities') {
        const centroids = {};
        MODEL.communities.forEach((c,i) => {
          const ids = new Set(simNodes.map(n=>n.id));
          const members = c.member_ids.filter(m=>ids.has(m));
          if (members.length>0) {
            const angle = (2*Math.PI*i)/Math.max(MODEL.communities.length,1);
            centroids[c.id]={x:width()/2+Math.cos(angle)*width()*.25,y:height()/2+Math.sin(angle)*height()*.25,members:new Set(members)};
          }
        });
        renderHulls(simNodes, centroids);
      } else {
        hullG.selectAll('*').remove();
      }
    });
  });

  document.getElementById('search').addEventListener('input', function() {
    const q = this.value.toLowerCase();
    nodeG.selectAll('g').style('opacity', d => {
      if (!q) return 1;
      return (d.label.toLowerCase().includes(q) || d.id.toLowerCase().includes(q)
        || (d.technology||'').toLowerCase().includes(q)) ? 1 : 0.15;
    });
  });

  document.getElementById('refreshBtn').addEventListener('click', () => {
    vscode.postMessage({type:'refresh'});
  });
}

// ---- drag ----
function drag(sim) {
  return d3.drag()
    .on('start', (ev,d) => { if(!ev.active) sim.alphaTarget(.3).restart(); d.fx=d.x; d.fy=d.y; })
    .on('drag', (ev,d) => { d.fx=ev.x; d.fy=ev.y; })
    .on('end', (ev,d) => { if(!ev.active) sim.alphaTarget(0); d.fx=null; d.fy=null; });
}

// ---- summary bar ----
function renderSummary() {
  const s = MODEL.summary;
  document.getElementById('summary').innerHTML =
    '<span><span class="val">'+s.total_nodes+'</span> nodes</span>'
    +'<span><span class="val">'+s.total_edges+'</span> edges</span>'
    +'<span>Health: <span class="val">'+s.health.replace(/_/g,' ')+'</span> ('+s.drift_score+')</span>'
    +'<span><span class="val">'+s.hotspot_count+'</span> hotspots</span>'
    +'<span><span class="val">'+s.cycle_count+'</span> cycles</span>'
    +'<span><span class="val">'+s.community_count+'</span> communities</span>';
}

init();
})();
<\/script>
</body>
</html>`;
}
