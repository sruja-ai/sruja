import { motion } from "framer-motion";

export const CanvasLoadingState = () => {
  return (
    <motion.div
      key="loading"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="canvas-empty"
    >
      <div className="canvas-empty-content">
        <p>Loading diagram...</p>
        <p className="canvas-empty-message">Converting model format</p>
      </div>
    </motion.div>
  );
};
