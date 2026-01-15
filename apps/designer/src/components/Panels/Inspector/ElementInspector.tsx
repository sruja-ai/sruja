import {
  Info,
  ArrowDownLeft, // Restored
  ArrowUpRight, // Restored
  ShieldCheck, // Restored
  FileText, // Restored
  FileCode, // Restored
  ArrowLeft,
  Plus,
} from "lucide-react";
import { Button } from "@sruja/ui";
import { useArchitectureStore, useSelectionStore, useUIStore } from "../../../stores";
import type { ElementDump, FqnRef, Requirement, ADR } from "@sruja/shared";

export function ElementInspector() {
  const model = useArchitectureStore((s) => s.model);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const selectNode = useSelectionStore((s) => s.selectNode);
  const setActiveTab = useUIStore((s) => s.setActiveTab);

  if (!selectedNodeId || !model) return null;

  // Find the selected node
  const node = model.elements?.[selectedNodeId] as ElementDump | undefined;
  if (!node) return null;

  const type = node.kind.charAt(0).toUpperCase() + node.kind.slice(1);

  // Helper to extract FQN
  const getFqn = (ref: FqnRef | string | undefined): string =>
    typeof ref === "object" && ref?.model ? ref.model : String(ref || "");

  // Dependency Analysis
  const allRelations = model.relations || [];

  const incoming = allRelations
    .filter((r) => getFqn(r.target) === selectedNodeId)
    .map((r) => ({
      relation: r,
      source: model.elements?.[getFqn(r.source)],
    }))
    .filter((x) => x.source);

  const outgoing = allRelations
    .filter((r) => getFqn(r.source) === selectedNodeId)
    .map((r) => ({
      relation: r,
      target: model.elements?.[getFqn(r.target)],
    }))
    .filter((x) => x.target);

  // Sruja extensions data
  const sruja = model.sruja || {};
  const allRequirements = sruja.requirements || [];
  const allADRs = sruja.adrs || [];

  const relatedRequirements = allRequirements.filter((req: Requirement) =>
    req.tags?.some((tag: string) => tag.toLowerCase() === node.id.toLowerCase())
  );

  const relatedADRs = allADRs.filter((adr: ADR) =>
    adr.tags?.some((tag: string) => tag.toLowerCase() === node.id.toLowerCase())
  );

  const getTypeIcon = (_nodeType: string) => <Info size={12} />;

  // Children

  return (
    <div className="inspector-content-wrapper">
      {/* Header Actions */}
      <div className="flex items-center justify-between mb-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => selectNode(null, "navigation")}
          className="text-gray-500 hover:text-gray-900 -ml-2"
        >
          <ArrowLeft size={16} className="mr-1" />
          Back to Project
        </Button>
        <Button variant="ghost" size="sm" onClick={() => setActiveTab("code")} title="View Source">
          <FileCode size={16} />
        </Button>
      </div>

      {/* Identity */}
      <section className="inspector-section">
        <div className="inspector-section-header">Identity</div>
        <div className="inspector-item">
          <div className="flex justify-between items-center mb-2">
            <span className="font-semibold text-gray-900 break-all">{node.title}</span>
            <span className="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs capitalize">
              {type}
            </span>
          </div>
          <div className="text-xs text-gray-500 font-mono bg-gray-100 p-1 rounded mb-2 block w-fit">
            {node.id}
          </div>
          {node.description && (
            <p className="text-sm text-gray-600 leading-relaxed">
              {typeof node.description === "string" ? node.description : ""}
            </p>
          )}
          {node.technology && (
            <div className="mt-2 text-xs">
              <span className="text-gray-500">Tech: </span>
              <span className="font-medium text-gray-700">{node.technology}</span>
            </div>
          )}
        </div>
      </section>

      {/* Smart Actions (Placeholder for now) */}
      {node.kind === "system" && (
        <Button variant="outline" className="w-full justify-start gap-2 border-dashed">
          <Plus size={14} />
          <span>Add Container</span>
        </Button>
      )}

      {/* Relations */}
      {(incoming.length > 0 || outgoing.length > 0) && (
        <section className="inspector-section">
          <div className="inspector-section-header">Dependencies</div>

          {incoming.length > 0 && (
            <div className="flex flex-col gap-1">
              <span className="text-xs font-medium text-gray-500 flex items-center gap-1">
                <ArrowDownLeft size={12} /> Incoming
              </span>
              {incoming.map((inc, idx) => (
                <div
                  key={idx}
                  className="inspector-item cursor-pointer hover:bg-gray-100 p-2"
                  onClick={() => selectNode(getFqn(inc.relation.source), "navigation")}
                >
                  <div className="flex items-center gap-2 text-sm font-medium">
                    {getTypeIcon((inc.source as ElementDump)?.kind || "")}
                    <span>{(inc.source as ElementDump)?.title || getFqn(inc.relation.source)}</span>
                  </div>
                  {(inc.relation.title || inc.relation.technology) && (
                    <span className="text-xs text-gray-500 block mt-1">
                      {inc.relation.title || inc.relation.technology}
                    </span>
                  )}
                </div>
              ))}
            </div>
          )}

          {outgoing.length > 0 && (
            <div className="flex flex-col gap-1 mt-2">
              <span className="text-xs font-medium text-gray-500 flex items-center gap-1">
                <ArrowUpRight size={12} /> Outgoing
              </span>
              {outgoing.map((out, idx) => (
                <div
                  key={idx}
                  className="inspector-item cursor-pointer hover:bg-gray-100 p-2"
                  onClick={() => selectNode(getFqn(out.relation.target), "navigation")}
                >
                  <div className="flex items-center gap-2 text-sm font-medium">
                    {getTypeIcon((out.target as ElementDump)?.kind || "")}
                    <span>{(out.target as ElementDump)?.title || getFqn(out.relation.target)}</span>
                  </div>
                  {(out.relation.title || out.relation.technology) && (
                    <span className="text-xs text-gray-500 block mt-1">
                      {out.relation.title || out.relation.technology}
                    </span>
                  )}
                </div>
              ))}
            </div>
          )}
        </section>
      )}

      {/* Related Items */}
      {(relatedRequirements.length > 0 || relatedADRs.length > 0) && (
        <section className="inspector-section">
          <div className="inspector-section-header">Governance</div>

          {relatedRequirements.map((req) => (
            <div key={req.id} className="inspector-item">
              <div className="flex justify-between items-start mb-1">
                <span className="text-sm font-medium flex items-center gap-1">
                  <ShieldCheck size={12} className="text-blue-500" />
                  {req.title || req.id}
                </span>
                {req.type && (
                  <span className="text-[10px] bg-blue-50 text-blue-600 px-1 rounded">
                    {req.type}
                  </span>
                )}
              </div>
            </div>
          ))}

          {relatedADRs.map((adr) => (
            <div key={adr.id} className="inspector-item border-l-2 border-l-purple-500">
              <div className="flex justify-between items-start mb-1">
                <span className="text-sm font-medium flex items-center gap-1">
                  <FileText size={12} className="text-purple-500" />
                  {adr.title || adr.id}
                </span>
                <span className="text-[10px] bg-purple-50 text-purple-600 px-1 rounded">
                  {adr.status || "Draft"}
                </span>
              </div>
            </div>
          ))}
        </section>
      )}
    </div>
  );
}
