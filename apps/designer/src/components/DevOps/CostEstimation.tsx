// apps/designer/src/components/DevOps/CostEstimation.tsx
import { useMemo } from "react";
import { DollarSign, Server, HardDrive, Network } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./CostEstimation.css";

interface CostBreakdown {
  compute: number;
  storage: number;
  network: number;
  total: number;
}

interface ResourceCost {
  type: string;
  cost: number;
  count: number;
}

// Default cost estimates (per month)
const DEFAULT_COSTS = {
  compute: {
    small: 50, // $50/month per small instance
    medium: 150,
    large: 400,
    default: 100,
  },
  storage: {
    gb: 0.1, // $0.10/GB/month
  },
  network: {
    gb: 0.12, // $0.12/GB/month
  },
};

export function CostEstimation() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const nodes = model.getNodes();

  // Estimate costs from architecture
  const costBreakdown = useMemo<CostBreakdown>(() => {
    if (!architectureModel) {
      return { compute: 0, storage: 0, network: 0, total: 0 };
    }

    let compute = 0;
    let storage = 0;
    let network = 0;

    // Estimate costs based on components
    for (const [nodeId, node] of nodes.entries()) {
      const nodeData = node as any;
      const tech = (nodeData.technology || "").toLowerCase();

      // Compute costs (estimate based on component type)
      if (nodeData.kind === "component" || nodeData.kind === "container") {
        // Estimate instance size based on component
        const size = tech.includes("large") ? "large" : tech.includes("small") ? "small" : "medium";
        compute += DEFAULT_COSTS.compute[size] || DEFAULT_COSTS.compute.default;
      }

      // Storage costs (estimate based on datastores)
      if (nodeData.kind === "datastore" || tech.includes("database") || tech.includes("db")) {
        // Estimate storage size (default 100GB per datastore)
        const estimatedGB = nodeData.metadata?.storage || 100;
        storage += estimatedGB * DEFAULT_COSTS.storage.gb;
      }

      // Network costs (estimate based on relations)
      // Simplified: assume each relation represents some network traffic
      const relations = model.getRelations();
      const componentRelations = relations.filter(
        (r: any) =>
          (r.source?.model === nodeId || r.target?.model === nodeId) &&
          r.source?.model !== r.target?.model
      );
      // Estimate 10GB/month per relation
      network += componentRelations.length * 10 * DEFAULT_COSTS.network.gb;
    }

    // Divide network by 2 to avoid double counting
    network = network / 2;

    return {
      compute: Math.round(compute),
      storage: Math.round(storage * 100) / 100,
      network: Math.round(network * 100) / 100,
      total: Math.round((compute + storage + network) * 100) / 100,
    };
  }, [nodes, architectureModel, model]);

  const resourceBreakdown = useMemo<ResourceCost[]>(() => {
    const resources: ResourceCost[] = [];
    const componentCount = Array.from(nodes.values()).filter(
      (n: any) => n.kind === "component" || n.kind === "container"
    ).length;
    const datastoreCount = Array.from(nodes.values()).filter(
      (n: any) => n.kind === "datastore"
    ).length;

    if (componentCount > 0) {
      resources.push({
        type: "Compute Instances",
        cost: costBreakdown.compute,
        count: componentCount,
      });
    }

    if (datastoreCount > 0) {
      resources.push({
        type: "Storage",
        cost: costBreakdown.storage,
        count: datastoreCount,
      });
    }

    if (costBreakdown.network > 0) {
      resources.push({
        type: "Network",
        cost: costBreakdown.network,
        count: 0, // Network doesn't have a simple count
      });
    }

    return resources;
  }, [nodes, costBreakdown]);

  return (
    <div className="cost-estimation">
      <div className="cost-estimation-header">
        <h3 className="cost-estimation-title">
          <DollarSign size={18} />
          Cost Estimation
        </h3>
        <div className="cost-estimation-total">
          <span className="cost-total-label">Total (monthly)</span>
          <span className="cost-total-value">${costBreakdown.total.toLocaleString()}</span>
        </div>
      </div>

      <div className="cost-estimation-content">
        <div className="cost-breakdown">
          <div className="cost-category">
            <div className="cost-category-header">
              <Server size={16} className="cost-category-icon" />
              <span className="cost-category-label">Compute</span>
              <span className="cost-category-value">${costBreakdown.compute.toLocaleString()}</span>
            </div>
            <div className="cost-category-bar">
              <div
                className="cost-bar-fill"
                style={{
                  width: `${(costBreakdown.compute / costBreakdown.total) * 100}%`,
                  backgroundColor: "#3b82f6",
                }}
              />
            </div>
          </div>

          <div className="cost-category">
            <div className="cost-category-header">
              <HardDrive size={16} className="cost-category-icon" />
              <span className="cost-category-label">Storage</span>
              <span className="cost-category-value">${costBreakdown.storage.toFixed(2)}</span>
            </div>
            <div className="cost-category-bar">
              <div
                className="cost-bar-fill"
                style={{
                  width: `${(costBreakdown.storage / costBreakdown.total) * 100}%`,
                  backgroundColor: "#22c55e",
                }}
              />
            </div>
          </div>

          <div className="cost-category">
            <div className="cost-category-header">
              <Network size={16} className="cost-category-icon" />
              <span className="cost-category-label">Network</span>
              <span className="cost-category-value">${costBreakdown.network.toFixed(2)}</span>
            </div>
            <div className="cost-category-bar">
              <div
                className="cost-bar-fill"
                style={{
                  width: `${(costBreakdown.network / costBreakdown.total) * 100}%`,
                  backgroundColor: "#f59e0b",
                }}
              />
            </div>
          </div>
        </div>

        <div className="cost-resources">
          <div className="cost-resources-title">Resource Breakdown</div>
          {resourceBreakdown.map((resource) => (
            <div key={resource.type} className="cost-resource-item">
              <span className="cost-resource-type">{resource.type}</span>
              {resource.count > 0 && (
                <span className="cost-resource-count">{resource.count} units</span>
              )}
              <span className="cost-resource-cost">${resource.cost.toLocaleString()}/month</span>
            </div>
          ))}
        </div>

        <div className="cost-estimation-note">
          <p className="cost-note-text">
            <strong>Note:</strong> Cost estimates are approximate and based on default pricing.
            Actual costs may vary based on provider, region, and usage patterns.
          </p>
        </div>
      </div>
    </div>
  );
}
