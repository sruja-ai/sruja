import { useEffect, useState } from "react";
import { FileText, ClipboardList, Plus, Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import { useArchitectureStore } from "../../stores/architectureStore";
import type { RequirementJSON, ADRJSON } from "../../types";
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
  const data = useArchitectureStore((s) => s.data);
  const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

  const allRequirements = data?.architecture?.requirements ?? [];
  const allAdrs = data?.architecture?.adrs ?? [];

  // Filter to show only items tagged with elements at this level
  const requirements = filterFn ? allRequirements.filter((r) => filterFn(r.tags)) : allRequirements;
  const adrs = filterFn ? allAdrs.filter((a) => filterFn(a.tags)) : allAdrs;

  const [activeTab, setActiveTab] = useState<"requirements" | "adrs">("requirements");
  const [expandedAdr, setExpandedAdr] = useState<string | null>(null);
  const [collapsed, setCollapsed] = useState(true);

  useEffect(() => {
    try {
      const key = `playground:govCollapsed:${levelLabel}`;
      const saved = window.localStorage.getItem(key);
      if (saved === "false") setCollapsed(false);
      else if (saved === "true") setCollapsed(true);
    } catch {}
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
    if (!reqId.trim() || !reqTitle.trim() || !data?.architecture) return;
    const newReq: RequirementJSON = {
      id: reqId.trim(),
      type: reqType,
      title: reqTitle.trim(),
      tags: reqTag ? [reqTag] : undefined,
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        requirements: [...(arch.architecture.requirements ?? []), newReq],
      },
    }));
    setReqId("");
    setReqTitle("");
    setReqTag("");
  };

  const removeRequirement = (id: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        requirements: (arch.architecture.requirements ?? []).filter((r) => r.id !== id),
      },
    }));
  };

  const addAdr = () => {
    if (!adrId.trim() || !adrTitle.trim() || !data?.architecture) return;
    const newAdr: ADRJSON = {
      id: adrId.trim(),
      title: adrTitle.trim(),
      status: adrStatus,
      context: adrContext.trim() || undefined,
      decision: adrDecision.trim() || undefined,
      tags: adrTag ? [adrTag] : undefined,
    };
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        adrs: [...(arch.architecture.adrs ?? []), newAdr],
      },
    }));
    setAdrId("");
    setAdrTitle("");
    setAdrStatus("proposed");
    setAdrContext("");
    setAdrDecision("");
    setAdrTag("");
  };

  const removeAdr = (id: string) => {
    updateArchitecture((arch) => ({
      ...arch,
      architecture: {
        ...arch.architecture,
        adrs: (arch.architecture.adrs ?? []).filter((a) => a.id !== id),
      },
    }));
  };

  // Don't render if no elements to tag
  if (elements.length === 0) {
    return null;
  }

  return (
    <div className="step-section governance-section">
      <div className="governance-toggle">
        <button
          className="gov-toggle-btn"
          onClick={() => {
            setCollapsed((v) => {
              const next = !v;
              try {
                const key = `playground:govCollapsed:${levelLabel}`;
                window.localStorage.setItem(key, next ? "true" : "false");
              } catch {}
              return next;
            });
          }}
        >
          {collapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
          <span>Governance</span>
        </button>
      </div>
      {!collapsed && (
        <div className="governance-mini-tabs">
          <button
            className={`gov-mini-tab ${activeTab === "requirements" ? "active" : ""}`}
            onClick={() => setActiveTab("requirements")}
          >
            <ClipboardList size={14} />
            Requirements
            <span className="count-badge">{requirements.length}</span>
          </button>
          <button
            className={`gov-mini-tab ${activeTab === "adrs" ? "active" : ""}`}
            onClick={() => setActiveTab("adrs")}
          >
            <FileText size={14} />
            ADRs
            <span className="count-badge">{adrs.length}</span>
          </button>
        </div>
      )}

      {/* Requirements */}
      {!collapsed && activeTab === "requirements" && (
        <>
          <p className="section-description">Define requirements linked to {levelLabel} elements</p>

          <div className="items-list">
            {requirements.map((req) => (
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
                <button className="item-remove" onClick={() => removeRequirement(req.id)}>
                  <Trash2 size={14} />
                </button>
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
            <div className="form-group">
              <label>Type</label>
              <select
                value={reqType}
                onChange={(e) => setReqType(e.target.value as "functional" | "nonfunctional")}
              >
                <option value="functional">Functional</option>
                <option value="nonfunctional">Non-Functional</option>
              </select>
            </div>
            <Input
              label="Title"
              value={reqTitle}
              onChange={(e) => setReqTitle(e.target.value)}
              placeholder="Must support 10k concurrent users"
            />
            <div className="form-group">
              <label>Link to</label>
              <select value={reqTag} onChange={(e) => setReqTag(e.target.value)}>
                <option value="">No link</option>
                {elements.map((el) => (
                  <option key={el.id} value={el.id}>
                    {el.id}
                  </option>
                ))}
              </select>
            </div>
            <Button
              variant="secondary"
              onClick={addRequirement}
              disabled={!reqId.trim() || !reqTitle.trim()}
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
            {adrs.map((adr) => (
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
                  <button
                    className="item-remove"
                    onClick={(e) => {
                      e.stopPropagation();
                      removeAdr(adr.id);
                    }}
                  >
                    <Trash2 size={14} />
                  </button>
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
              <div className="form-group">
                <label>Status</label>
                <select value={adrStatus} onChange={(e) => setAdrStatus(e.target.value)}>
                  <option value="proposed">Proposed</option>
                  <option value="accepted">Accepted</option>
                  <option value="deprecated">Deprecated</option>
                </select>
              </div>
              <div className="form-group">
                <label>Link to</label>
                <select value={adrTag} onChange={(e) => setAdrTag(e.target.value)}>
                  <option value="">No link</option>
                  {elements.map((el) => (
                    <option key={el.id} value={el.id}>
                      {el.id}
                    </option>
                  ))}
                </select>
              </div>
            </div>
            <div className="form-row">
              <div className="form-group textarea-group-mini">
                <label>Context</label>
                <input
                  type="text"
                  value={adrContext}
                  onChange={(e) => setAdrContext(e.target.value)}
                  placeholder="Why this decision?"
                />
              </div>
              <div className="form-group textarea-group-mini">
                <label>Decision</label>
                <input
                  type="text"
                  value={adrDecision}
                  onChange={(e) => setAdrDecision(e.target.value)}
                  placeholder="What was decided?"
                />
              </div>
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
