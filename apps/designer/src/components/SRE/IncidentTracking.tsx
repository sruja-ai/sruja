// apps/designer/src/components/SRE/IncidentTracking.tsx
import { useState } from "react";
import { AlertTriangle, Clock, CheckCircle } from "lucide-react";
import "./IncidentTracking.css";

type IncidentStatus = "open" | "investigating" | "resolved";
type IncidentSeverity = "critical" | "high" | "medium" | "low";

interface Incident {
  id: string;
  title: string;
  service: string;
  severity: IncidentSeverity;
  status: IncidentStatus;
  createdAt: string;
  resolvedAt?: string;
  duration?: number; // in minutes
}

export function IncidentTracking() {
  const [incidents] = useState<Incident[]>([
    {
      id: "inc-1",
      title: "Payment Service Degradation",
      service: "PaymentService",
      severity: "high",
      status: "resolved",
      createdAt: "2 hours ago",
      resolvedAt: "30 minutes ago",
      duration: 90,
    },
    {
      id: "inc-2",
      title: "Search API Timeout",
      service: "SearchService",
      severity: "medium",
      status: "investigating",
      createdAt: "1 hour ago",
    },
    {
      id: "inc-3",
      title: "Database Connection Pool Exhausted",
      service: "OrderService",
      severity: "critical",
      status: "open",
      createdAt: "15 minutes ago",
    },
  ]);

  const getSeverityColor = (severity: IncidentSeverity) => {
    switch (severity) {
      case "critical":
        return "#ef4444";
      case "high":
        return "#f59e0b";
      case "medium":
        return "#fbbf24";
      case "low":
        return "#22c55e";
    }
  };

  const getStatusIcon = (status: IncidentStatus) => {
    switch (status) {
      case "open":
        return <AlertTriangle size={14} className="icon-open" />;
      case "investigating":
        return <Clock size={14} className="icon-investigating" />;
      case "resolved":
        return <CheckCircle size={14} className="icon-resolved" />;
    }
  };

  const getStatusColor = (status: IncidentStatus) => {
    switch (status) {
      case "open":
        return "#ef4444";
      case "investigating":
        return "#f59e0b";
      case "resolved":
        return "#22c55e";
    }
  };

  const openIncidents = incidents.filter((i) => i.status !== "resolved").length;
  const criticalIncidents = incidents.filter(
    (i) => i.severity === "critical" && i.status !== "resolved"
  ).length;

  return (
    <div className="incident-tracking">
      <div className="incident-tracking-header">
        <h3 className="incident-tracking-title">
          <AlertTriangle size={18} />
          Incident Tracking
        </h3>
        <div className="incident-tracking-stats">
          {openIncidents > 0 && <span className="incident-stat">{openIncidents} Open</span>}
          {criticalIncidents > 0 && (
            <span className="incident-stat incident-critical">{criticalIncidents} Critical</span>
          )}
        </div>
      </div>

      <div className="incident-tracking-content">
        {incidents.length === 0 ? (
          <div className="incident-tracking-empty">
            <CheckCircle size={32} className="empty-icon" />
            <p>No active incidents.</p>
            <p className="incident-tracking-empty-subtitle">All systems operational.</p>
          </div>
        ) : (
          <div className="incident-list">
            {incidents.map((incident) => (
              <div key={incident.id} className={`incident-item incident-${incident.severity}`}>
                <div className="incident-item-header">
                  <div className="incident-item-info">
                    <div className="incident-item-title">{incident.title}</div>
                    <div className="incident-item-service">{incident.service}</div>
                  </div>
                  <div className="incident-item-badges">
                    <span
                      className="incident-severity-badge"
                      style={{ backgroundColor: getSeverityColor(incident.severity) }}
                    >
                      {incident.severity.toUpperCase()}
                    </span>
                    <div
                      className="incident-status-badge"
                      style={{ color: getStatusColor(incident.status) }}
                    >
                      {getStatusIcon(incident.status)}
                      <span>{incident.status}</span>
                    </div>
                  </div>
                </div>
                <div className="incident-item-meta">
                  <span className="incident-meta-item">Created: {incident.createdAt}</span>
                  {incident.resolvedAt && (
                    <span className="incident-meta-item">Resolved: {incident.resolvedAt}</span>
                  )}
                  {incident.duration && (
                    <span className="incident-meta-item">Duration: {incident.duration} min</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
