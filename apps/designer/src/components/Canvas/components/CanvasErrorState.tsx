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
            className="likec4-canvas-empty"
        >
            <div className="likec4-canvas-empty-content">
                <p>Diagram rendering failed</p>
                <p className="likec4-canvas-empty-message">{error}</p>
            </div>
        </motion.div>
    );
};
