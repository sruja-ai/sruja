import { motion } from "framer-motion";

interface CanvasErrorStateProps {
  error: string;
}

export const CanvasErrorState = ({ error }: CanvasErrorStateProps) => {
  return (
    <motion.div
      key="error"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="canvas-empty"
    >
      <div className="canvas-empty-content">
        <p>Diagram rendering failed</p>
        <p className="canvas-empty-message">{error}</p>
      </div>
    </motion.div>
  );
};
