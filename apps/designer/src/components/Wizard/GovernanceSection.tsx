import { useEffect, useState } from "react";
import { FileText, ClipboardList, Plus, Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { Button, Input, Select } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import { deduplicateRequirements } from "../../utils/deduplicateRequirements";
import type { RequirementDump, ADRDump } from "@sruja/shared";
import "./WizardSteps.css";

interface ElementOption {
  id: string;
  label: string;
}

interface GovernanceSectionProps {
  /** Elements available for tagging (at this C4 level) */
  elements: ElementOption[];
  /** C4 level label for display */
  levelLabel: string;
  /** Filter function to show only relevant items */
  filterFn?: (tags: string[] | undefined) => boolean;
}

export function GovernanceSection({ elements, levelLabel, filterFn }: GovernanceSectionProps) {
  const data = useArchitectureStore((s) => s.likec4Model);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const sruja = (data as any)?.sruja ?? {}; // Dump structure for Sruja extensions
  const allRequirements: RequirementDump[] = sruja.requirements ?? [];
  const allAdrs: ADRDump[] = sruja.adrs ?? [];

  // Deduplicate requirements first, then filter
  const uniqueRequirements = deduplicateRequirements(allRequirements as any); // Cast to any because deduplicate might expect legacy types or just id/title

  // Filter to show only items tagged with elements at this level
  const requirements = filterFn ? uniqueRequirements.filter((r: any) => filterFn(r.tags)) : uniqueRequirements;
  const adrs = filterFn ? allAdrs.filter((a: any) => filterFn(a.tags)) : allAdrs; // adr.tags might be missing in interface

  const [activeTab, setActiveTab] = useState<"requirements" | "adrs">("requirements");
  const [expandedAdr, setExpandedAdr] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState(true);

  useEffect(() => {
    try {
      const key = `playground:govCollapsed:${levelLabel}`;
      const saved = window.localStorage.getItem(key);
      if (saved === "false") setCollapsed(false);
      else if (saved === "true") setCollapsed(true);
    } catch { }
  }, [levelLabel]);

  // Requirement form
  const [reqId, setReqId] = useState("");
  const [reqType, setReqType] = useState<"functional" | "nonfunctional">("functional");
  const [reqTitle, setReqTitle] = useState("");
  const [reqTag, setReqTag] = useState("");

  // ADR form
  const [adrId, setAdrId] = useState("");
  const [adrTitle, setAdrTitle] = useState("");
  const [adrStatus, setAdrStatus] = useState("proposed");
  const [adrContext, setAdrContext] = useState("");
  const [adrDecision, setAdrDecision] = useState("");
  const [adrTag, setAdrTag] = useState("");

  const addRequirement = () => {
    if (!reqId.trim() || !reqTitle.trim() || !data) return;
    if (allRequirements.some((r) => r.id === reqId.trim())) return;
    const newReq: any = {
      id: reqId.trim(),
      type: reqType,
      title: reqTitle.trim(),
      tags: reqTag ? [reqTag] : undefined,
    };

    updateArchitecture((model) => {
      const currentSruja = (model as any).sruja || {};
      const currentReqs = currentSruja.requirements || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          requirements: [...currentReqs, newReq]
        }
      };
    });

    setReqId("");
    setReqTitle("");
    setReqTag("");
  };

  const removeRequirement = (id: string) => {
    updateArchitecture((model) => {
      const currentSruja = (model as any).sruja || {};
      const currentReqs = currentSruja.requirements || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          requirements: currentReqs.filter((r: any) => r.id !== id)
        }
      };
    });
  };

  const addAdr = () => {
    if (!adrId.trim() || !adrTitle.trim() || !data) return;
    const newAdr: any = {
      id: adrId.trim(),
      title: adrTitle.trim(),
      status: adrStatus,
      context: adrContext.trim() || undefined,
      decision: adrDecision.trim() || undefined,
      tags: adrTag ? [adrTag] : undefined,
    };

    updateArchitecture((model) => {
      const currentSruja = (model as any).sruja || {};
      const currentAdrs = currentSruja.adrs || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          adrs: [...currentAdrs, newAdr]
        }
      };
    });

    setAdrId("");
    setAdrTitle("");
    setAdrStatus("proposed");
    setAdrContext("");
    setAdrDecision("");
    setAdrTag("");
  };

  const removeAdr = (id: string) => {
    updateArchitecture((model) => {
      const currentSruja = (model as any).sruja || {};
      const currentAdrs = currentSruja.adrs || [];
      return {
        ...model,
        sruja: {
          ...currentSruja,
          adrs: currentAdrs.filter((a: any) => a.id !== id)
        }
      };
    });
  };

  // Don't render if no elements to tag
  if (elements.length === 0) {
    return null;
  }

  return (
    <div className="step-section governance-section">
      <div className="governance-toggle">
        <Button
          variant="ghost"
          size="sm"
          className="gov-toggle-btn"
          onClick={() => {
            setCollapsed((v) => {
              const next = !v;
              try {
                const key = `playground:govCollapsed:${levelLabel}`;
                window.localStorage.setItem(key, next ? "true" : "false");
              } catch { }
              return next;
            });
          }}
        >
          {collapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
          <span>Governance</span>
        </Button>
      </div>
      {!collapsed && (
        <div className="governance-mini-tabs">
          <Button
            variant={activeTab === "requirements" ? "primary" : "ghost"}
            size="sm"
            className={`gov-mini-tab ${activeTab === "requirements" ? "active" : ""}`}
            onClick={() => setActiveTab("requirements")}
          >
            <ClipboardList size={14} />
            Requirements
            <span className="count-badge">{requirements.length}</span>
          </Button>
          <Button
            variant={activeTab === "adrs" ? "primary" : "ghost"}
            size="sm"
            className={`gov-mini-tab ${activeTab === "adrs" ? "active" : ""}`}
            onClick={() => setActiveTab("adrs")}
          >
            <FileText size={14} />
            ADRs
            <span className="count-badge">{adrs.length}</span>
          </Button>
        </div>
      )}

      {/* Requirements */}
      {!collapsed && activeTab === "requirements" && (
        <>
          <p className="section-description">Define requirements linked to {levelLabel} elements</p>

          <div className="items-list">
            {requirements.map((req: any) => (
              <div key={req.id} className="item-card">
                <span
                  className={`item-type ${req.type === "functional" ? "functional" : "non-functional"}`}
                >
                  {req.type === "functional" ? "FR" : "NFR"}
                </span>
                <div className="item-info">
                  <span className="item-id">{req.id}</span>
                  <span className="item-label">{req.title}</span>
                </div>
                {req.tags?.[0] && <span className="item-tag">{req.tags[0]}</span>}
                <Button variant="ghost" size="sm" className="item-remove" onClick={() => removeRequirement(req.id)}>
                  <Trash2 size={14} />
                </Button>
              </div>
            ))}
          </div>

          <div className="add-form governance-form">
            <Input
              label="ID"
              value={reqId}
              onChange={(e) => setReqId(e.target.value.replace(/\s/g, ""))}
              placeholder="REQ001"
            />
            <Select
              label="Type"
              value={reqType}
              onChange={(value) => setReqType((value || "functional") as "functional" | "nonfunctional")}
              data={[
                { value: "functional", label: "Functional" },
                { value: "nonfunctional", label: "Non-Functional" },
              ]}
            />
            <Input
              label="Title"
              value={reqTitle}
              onChange={(e) => setReqTitle(e.target.value)}
              placeholder="Must support 10k concurrent users"
            />
            <Select
              label="Link to"
              value={reqTag}
              onChange={(value) => setReqTag(value || "")}
              placeholder="No link"
              data={elements.map((el) => ({ value: el.id, label: el.id }))}
            />
            <Button
              variant="secondary"
              onClick={addRequirement}
              disabled={
                !reqId.trim() || !reqTitle.trim() || allRequirements.some((r) => r.id === reqId.trim())
              }
              title={
                allRequirements.some((r) => r.id === reqId.trim())
                  ? "Duplicate requirement ID"
                  : undefined
              }
            >
              <Plus size={16} />
            </Button>
          </div>
        </>
      )}

      {/* ADRs */}
      {!collapsed && activeTab === "adrs" && (
        <>
          <p className="section-description">Document architectural decisions for {levelLabel}</p>

          <div className="items-list">
            {adrs.map((adr: any) => (
              <div key={adr.id} className="item-card adr-card-mini">
                <div
                  className="adr-header-mini"
                  onClick={() => setExpandedAdr(expandedAdr === adr.id ? null : adr.id)}
                >
                  <span className={`adr-status ${adr.status}`}>{adr.status}</span>
                  <span className="item-id">{adr.id}</span>
                  <span className="item-label">{adr.title}</span>
                  {adr.tags?.[0] && <span className="item-tag">{adr.tags[0]}</span>}
                  {expandedAdr === adr.id ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                  <Button
                    variant="ghost"
                    size="sm"
                    className="item-remove"
                    onClick={(e) => {
                      e.stopPropagation();
                      removeAdr(adr.id);
                    }}
                  >
                    <Trash2 size={14} />
                  </Button>
                </div>
                {expandedAdr === adr.id && (
                  <div className="adr-details-mini">
                    {adr.context && (
                      <div className="adr-field">
                        <strong>Context:</strong> {adr.context}
                      </div>
                    )}
                    {adr.decision && (
                      <div className="adr-field">
                        <strong>Decision:</strong> {adr.decision}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>

          <div className="add-form governance-form adr-mini-form">
            <div className="form-row">
              <Input
                label="ID"
                value={adrId}
                onChange={(e) => setAdrId(e.target.value.replace(/\s/g, ""))}
                placeholder="ADR001"
              />
              <Input
                label="Title"
                value={adrTitle}
                onChange={(e) => setAdrTitle(e.target.value)}
                placeholder="Use PostgreSQL for orders"
              />
              <Select
                label="Status"
                value={adrStatus}
                onChange={(value) => setAdrStatus(value || "proposed")}
                data={[
                  { value: "proposed", label: "Proposed" },
                  { value: "accepted", label: "Accepted" },
                  { value: "deprecated", label: "Deprecated" },
                ]}
              />
              <Select
                label="Link to"
                value={adrTag}
                onChange={(value) => setAdrTag(value || "")}
                placeholder="No link"
                data={elements.map((el) => ({ value: el.id, label: el.id }))}
              />
            </div>
            <div className="form-row">
              <Input
                label="Context"
                value={adrContext}
                onChange={(e) => setAdrContext(e.target.value)}
                placeholder="Why this decision?"
              />
              <Input
                label="Decision"
                value={adrDecision}
                onChange={(e) => setAdrDecision(e.target.value)}
                placeholder="What was decided?"
              />
              <Button
                variant="secondary"
                onClick={addAdr}
                disabled={!adrId.trim() || !adrTitle.trim()}
              >
                <Plus size={16} />
              </Button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
