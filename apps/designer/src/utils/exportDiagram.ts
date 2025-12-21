// apps/playground/src/utils/exportDiagram.ts
import type { ReactFlowInstance, Node } from "@xyflow/react";
import type { C4NodeData } from "../types";

/**
 * Export diagram as PNG using html2canvas
 */
export async function exportAsPNG(
  _reactFlowInstance: ReactFlowInstance<Node<C4NodeData>>,
  containerElement: HTMLElement,
  filename: string = "diagram.png"
): Promise<void> {
  try {
    // Try to find React Flow viewport or LikeC4 diagram container
    let viewportElement = containerElement.querySelector(
      ".react-flow__viewport"
    ) as HTMLElement;
    
    // Fallback to LikeC4 diagram container
    if (!viewportElement) {
      viewportElement = containerElement.querySelector(
        ".likec4-diagram-container, .likec4-canvas"
      ) as HTMLElement;
    }
    
    if (!viewportElement) {
      throw new Error("Diagram viewport not found (neither React Flow nor LikeC4)");
    }

    // Use html2canvas if available, otherwise use canvas API
    const html2canvas = await import("html2canvas").catch(() => null);

    if (html2canvas?.default) {
      // Use html2canvas for better quality
      const canvas = await html2canvas.default(viewportElement, {
        backgroundColor:
          getComputedStyle(document.documentElement).getPropertyValue("--bg-primary") || "#ffffff",
        scale: 2, // Higher quality
        useCORS: true,
        logging: false,
      });

      // Convert to blob and download
      canvas.toBlob((blob) => {
        if (blob) {
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;
          a.download = filename;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }
      }, "image/png");
    } else {
      // Fallback: Create a simple canvas representation
      // This is a basic fallback - html2canvas is recommended
      throw new Error("html2canvas not available. Please install html2canvas for PNG export.");
    }
  } catch (error) {
    console.error("Failed to export PNG:", error);
    throw error;
  }
}

/**
 * Export diagram as SVG
 */
export async function exportAsSVG(
  reactFlowInstance: ReactFlowInstance<Node<C4NodeData>>,
  _containerElement: HTMLElement,
  filename: string = "diagram.svg"
): Promise<void> {
  try {
    const nodes = reactFlowInstance.getNodes();
    const edges = reactFlowInstance.getEdges();

    // Get bounding box of all nodes
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    nodes.forEach((node) => {
      const x = node.position.x;
      const y = node.position.y;
      const width = (node.width as number) || 200;
      const height = (node.height as number) || 100;

      minX = Math.min(minX, x);
      minY = Math.min(minY, y);
      maxX = Math.max(maxX, x + width);
      maxY = Math.max(maxY, y + height);
    });

    // Add padding
    const padding = 50;
    const width = maxX - minX + padding * 2;
    const height = maxY - minY + padding * 2;

    // Get background color
    const bgColor =
      getComputedStyle(document.documentElement).getPropertyValue("--bg-primary") || "#ffffff";

    // Create SVG
    let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="${minX - padding} ${minY - padding} ${width} ${height}">\n`;
    svg += `<rect width="${width}" height="${height}" fill="${bgColor}"/>\n`;

    // Add edges first (so they appear behind nodes)
    edges.forEach((edge) => {
      if (edge.source && edge.target) {
        const sourceNode = nodes.find((n) => n.id === edge.source);
        const targetNode = nodes.find((n) => n.id === edge.target);

        if (sourceNode && targetNode) {
          const sx =
            ((sourceNode.position?.x as number) || 0) + ((sourceNode.width as number) || 200) / 2;
          const sy =
            ((sourceNode.position?.y as number) || 0) + ((sourceNode.height as number) || 100) / 2;
          const tx =
            ((targetNode.position?.x as number) || 0) + ((targetNode.width as number) || 200) / 2;
          const ty =
            ((targetNode.position?.y as number) || 0) + ((targetNode.height as number) || 100) / 2;

          svg += `<line x1="${sx}" y1="${sy}" x2="${tx}" y2="${ty}" stroke="#666" stroke-width="2" marker-end="url(#arrowhead)"/>\n`;
        }
      }
    });

    // Add arrow marker definition
    svg += `<defs><marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto"><polygon points="0 0, 10 3, 0 6" fill="#666"/></marker></defs>\n`;

    // Add nodes
    nodes.forEach((node) => {
      const x = (node.position?.x as number) || 0;
      const y = (node.position?.y as number) || 0;
      const width = (node.width as number) || 200;
      const height = (node.height as number) || 100;
      const data = node.data as C4NodeData;

      // Get node color based on type
      let fillColor = "#e2e8f0";
      switch (data.type) {
        case "person":
          fillColor = "#3b82f6";
          break;
        case "system":
          fillColor = "#10b981";
          break;
        case "container":
          fillColor = "#f59e0b";
          break;
        case "component":
          fillColor = "#8b5cf6";
          break;
      }

      svg += `<rect x="${x}" y="${y}" width="${width}" height="${height}" fill="${fillColor}" stroke="#333" stroke-width="2" rx="4"/>\n`;
      svg += `<text x="${x + width / 2}" y="${y + 20}" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#000">${data.label || node.id}</text>\n`;
      if (data.description) {
        svg += `<text x="${x + width / 2}" y="${y + 40}" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#666">${data.description}</text>\n`;
      }
    });

    svg += "</svg>";

    // Download SVG
    const blob = new Blob([svg], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error("Failed to export SVG:", error);
    throw error;
  }
}
