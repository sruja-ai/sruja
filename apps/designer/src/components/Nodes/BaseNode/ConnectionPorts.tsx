import { Handle, Position } from "@xyflow/react";

interface ConnectionPortsProps {
  // If not provided, defaults to all sides
  sides?: ("top" | "right" | "bottom" | "left")[];
  isVisible?: boolean; // For debugging handles
}

export function ConnectionPorts({
  sides = ["top", "right", "bottom", "left"],
  isVisible = false,
}: ConnectionPortsProps) {
  const style = {
    opacity: isVisible ? 1 : 0,
    width: 8,
    height: 8,
    background: "#777",
  };

  // Position offset for handles to ensure they are on the border but don't cause gaps
  const OFFSET = -4;

  return (
    <>
      {sides.includes("top") && (
        <>
          <Handle
            type="target"
            position={Position.Top}
            id="t"
            className="c4-port"
            style={{ ...style, top: OFFSET }}
          />
          <Handle
            type="source"
            position={Position.Top}
            id="st"
            className="c4-port"
            style={{ ...style, top: OFFSET }}
          />
        </>
      )}
      {sides.includes("right") && (
        <>
          <Handle
            type="target"
            position={Position.Right}
            id="r"
            className="c4-port"
            style={{ ...style, right: OFFSET }}
          />
          <Handle
            type="source"
            position={Position.Right}
            id="sr"
            className="c4-port"
            style={{ ...style, right: OFFSET }}
          />
        </>
      )}
      {sides.includes("bottom") && (
        <>
          <Handle
            type="target"
            position={Position.Bottom}
            id="b"
            className="c4-port"
            style={{ ...style, bottom: OFFSET }}
          />
          <Handle
            type="source"
            position={Position.Bottom}
            id="sb"
            className="c4-port"
            style={{ ...style, bottom: OFFSET }}
          />
        </>
      )}
      {sides.includes("left") && (
        <>
          <Handle
            type="target"
            position={Position.Left}
            id="l"
            className="c4-port"
            style={{ ...style, left: OFFSET }}
          />
          <Handle
            type="source"
            position={Position.Left}
            id="sl"
            className="c4-port"
            style={{ ...style, left: OFFSET }}
          />
        </>
      )}
    </>
  );
}
