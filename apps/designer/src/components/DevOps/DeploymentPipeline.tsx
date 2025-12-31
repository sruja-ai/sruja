// apps/designer/src/components/DevOps/DeploymentPipeline.tsx
import { useState } from "react";
import { GitBranch, CheckCircle, XCircle, Clock, Plus } from "lucide-react";
import { Button } from "@sruja/ui";
import "./DeploymentPipeline.css";

interface PipelineStage {
  id: string;
  name: string;
  status: "success" | "failed" | "running" | "pending";
  duration?: number;
}

interface Pipeline {
  id: string;
  service: string;
  stages: PipelineStage[];
  lastRun?: string;
}

export function DeploymentPipeline() {
  const [pipelines] = useState<Pipeline[]>([
    {
      id: "pipeline-1",
      service: "OrderService",
      stages: [
        { id: "build", name: "Build", status: "success", duration: 120 },
        { id: "test", name: "Test", status: "success", duration: 300 },
        { id: "deploy", name: "Deploy", status: "success", duration: 180 },
      ],
      lastRun: "2 hours ago",
    },
    {
      id: "pipeline-2",
      service: "PaymentService",
      stages: [
        { id: "build", name: "Build", status: "success", duration: 90 },
        { id: "test", name: "Test", status: "running", duration: 150 },
        { id: "deploy", name: "Deploy", status: "pending" },
      ],
      lastRun: "30 minutes ago",
    },
  ]);

  const getStageIcon = (status: PipelineStage["status"]) => {
    switch (status) {
      case "success":
        return <CheckCircle size={14} className="icon-success" />;
      case "failed":
        return <XCircle size={14} className="icon-failed" />;
      case "running":
        return <Clock size={14} className="icon-running" />;
      default:
        return <Clock size={14} className="icon-pending" />;
    }
  };

  const getStageColor = (status: PipelineStage["status"]) => {
    switch (status) {
      case "success":
        return "#22c55e";
      case "failed":
        return "#ef4444";
      case "running":
        return "#3b82f6";
      default:
        return "#6b7280";
    }
  };

  return (
    <div className="deployment-pipeline">
      <div className="deployment-pipeline-header">
        <h3 className="deployment-pipeline-title">
          <GitBranch size={18} />
          Deployment Pipelines
        </h3>
        <Button variant="secondary" size="sm">
          <Plus size={14} />
          Add Pipeline
        </Button>
      </div>

      <div className="deployment-pipeline-content">
        {pipelines.length === 0 ? (
          <div className="deployment-pipeline-empty">
            <p>No deployment pipelines configured.</p>
            <p className="deployment-pipeline-empty-subtitle">
              Add pipelines to track CI/CD status per service.
            </p>
          </div>
        ) : (
          <div className="deployment-pipeline-list">
            {pipelines.map((pipeline) => (
              <div key={pipeline.id} className="pipeline-item">
                <div className="pipeline-item-header">
                  <div className="pipeline-service">{pipeline.service}</div>
                  {pipeline.lastRun && (
                    <div className="pipeline-last-run">Last run: {pipeline.lastRun}</div>
                  )}
                </div>
                <div className="pipeline-stages">
                  {pipeline.stages.map((stage, index) => (
                    <div key={stage.id} className="pipeline-stage">
                      <div className="pipeline-stage-header">
                        {getStageIcon(stage.status)}
                        <span className="pipeline-stage-name">{stage.name}</span>
                        {stage.duration && (
                          <span className="pipeline-stage-duration">
                            {Math.floor(stage.duration / 60)}m {stage.duration % 60}s
                          </span>
                        )}
                      </div>
                      <div className="pipeline-stage-bar">
                        <div
                          className="pipeline-stage-fill"
                          style={{
                            width: stage.status === "pending" ? "0%" : "100%",
                            backgroundColor: getStageColor(stage.status),
                          }}
                        />
                      </div>
                      {index < pipeline.stages.length - 1 && (
                        <div className="pipeline-stage-connector" />
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
