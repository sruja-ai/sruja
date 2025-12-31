// apps/designer/src/components/DevOps/InfrastructureMap.tsx
import { useMemo } from "react";
import { Server, Globe, Database, Box } from "lucide-react";
import { getArchitectureModel } from "../../models/ArchitectureModel";
import "./InfrastructureMap.css";

interface InfrastructureNode {
  id: string;
  type: "region" | "cluster" | "node" | "component";
  name: string;
  children?: InfrastructureNode[];
  componentId?: string;
  technology?: string;
  metadata?: any;
}

export function InfrastructureMap() {
  const model = getArchitectureModel();
  const architectureModel = model.getModel();
  const nodes = model.getNodes();

  // Build infrastructure topology from deployments and components
  const infrastructure = useMemo<InfrastructureNode[]>(() => {
    if (!architectureModel) return [];

    const topology: InfrastructureNode[] = [];
    const regions = new Map<string, InfrastructureNode>();

    // Extract deployment information from model elements
    // Deployments might be stored in elements or as separate deployment nodes
    for (const [, node] of nodes.entries()) {
      const nodeData = node as any;

      // Check if this is a deployment node
      if (nodeData.kind === "deployment" || nodeData.deployment) {
        const deploymentData = nodeData.deployment || nodeData;
        const region = deploymentData.region || deploymentData.label || "default";
        const cluster = deploymentData.cluster || deploymentData.id || "default-cluster";

        if (!regions.has(region)) {
          regions.set(region, {
            id: region,
            type: "region",
            name: region,
            children: [],
          });
        }

        const regionNode = regions.get(region)!;

        // Find or create cluster
        let clusterNode = regionNode.children?.find((c) => c.id === cluster);
        if (!clusterNode) {
          clusterNode = {
            id: cluster,
            type: "cluster",
            name: cluster,
            children: [],
          };
          regionNode.children!.push(clusterNode);
        }

        // Add components deployed to this cluster
        const componentId = deploymentData.componentId || deploymentData.containerInstance;
        if (componentId && nodes.has(componentId)) {
          const component = nodes.get(componentId) as any;
          clusterNode.children!.push({
            id: componentId,
            type: "component",
            name: componentId,
            componentId,
            technology: component.technology,
            metadata: component,
          });
        }
      }
    }

    // If no deployments found, show components grouped by technology
    if (topology.length === 0 && regions.size === 0) {
      const componentsByTech = new Map<string, InfrastructureNode[]>();

      for (const [nodeId, node] of nodes.entries()) {
        const nodeData = node as any;
        const tech = nodeData.technology || "unknown";

        if (!componentsByTech.has(tech)) {
          componentsByTech.set(tech, []);
        }

        componentsByTech.get(tech)!.push({
          id: nodeId,
          type: "component",
          name: nodeId,
          componentId: nodeId,
          technology: tech,
          metadata: nodeData,
        });
      }

      // Create a default region with clusters by technology
      const defaultRegion: InfrastructureNode = {
        id: "default-region",
        type: "region",
        name: "Default Region",
        children: [],
      };

      for (const [tech, components] of componentsByTech.entries()) {
        defaultRegion.children!.push({
          id: `cluster-${tech}`,
          type: "cluster",
          name: `${tech} Cluster`,
          children: components,
        });
      }

      if (defaultRegion.children!.length > 0) {
        topology.push(defaultRegion);
      }
    } else if (regions.size > 0) {
      topology.push(...Array.from(regions.values()));
    }

    return topology;
  }, [nodes, architectureModel]);

  const getNodeIcon = (type: InfrastructureNode["type"]) => {
    switch (type) {
      case "region":
        return <Globe size={16} />;
      case "cluster":
        return <Server size={16} />;
      case "component":
        return <Box size={16} />;
      default:
        return <Database size={16} />;
    }
  };

  const renderNode = (node: InfrastructureNode, depth: number = 0) => {
    const indent = depth * 20;

    return (
      <div key={node.id} className="infrastructure-node" style={{ paddingLeft: `${indent}px` }}>
        <div className="infrastructure-node-header">
          <div className="infrastructure-node-icon">{getNodeIcon(node.type)}</div>
          <div className="infrastructure-node-content">
            <div className="infrastructure-node-name">{node.name}</div>
            {node.technology && <div className="infrastructure-node-tech">{node.technology}</div>}
          </div>
        </div>
        {node.children && node.children.length > 0 && (
          <div className="infrastructure-node-children">
            {node.children.map((child) => renderNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  if (infrastructure.length === 0) {
    return (
      <div className="infrastructure-map">
        <div className="infrastructure-map-header">
          <h3 className="infrastructure-map-title">
            <Server size={18} />
            Infrastructure Topology
          </h3>
        </div>
        <div className="infrastructure-map-empty">
          <p>No infrastructure defined yet.</p>
          <p className="infrastructure-map-empty-subtitle">
            Add deployment nodes to visualize infrastructure topology.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="infrastructure-map">
      <div className="infrastructure-map-header">
        <h3 className="infrastructure-map-title">
          <Server size={18} />
          Infrastructure Topology
        </h3>
      </div>
      <div className="infrastructure-map-content">
        {infrastructure.map((node) => renderNode(node))}
      </div>
    </div>
  );
}
