import { X, ZoomIn, ZoomOut } from 'lucide-react';
import type { ExpandedMermaid } from '../types';

interface MermaidExpandedModalProps {
  expandedMermaid: ExpandedMermaid;
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onClose: () => void;
}

export function MermaidExpandedModal({
  expandedMermaid,
  zoom,
  onZoomIn,
  onZoomOut,
  onClose,
}: MermaidExpandedModalProps) {
  return (
    <div 
      className="mermaid-expanded-overlay"
      onClick={onClose}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'var(--overlay-scrim)',
        zIndex: 9999,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '10px',
        cursor: 'pointer'
      }}
    >
      <div 
        onClick={(e) => e.stopPropagation()}
        style={{
          backgroundColor: 'var(--color-background)',
          borderRadius: '8px',
          padding: '24px',
          width: '98vw',
          height: '98vh',
          overflow: 'auto',
          position: 'relative',
          cursor: 'default',
          boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
          display: 'flex',
          flexDirection: 'column'
        }}
      >
        {/* Zoom Controls */}
        <div
          style={{
            position: 'absolute',
            top: '12px',
            right: '12px',
            display: 'flex',
            gap: '8px',
            alignItems: 'center',
            zIndex: 10
          }}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              onZoomIn();
            }}
            style={{
              background: 'var(--color-background)',
              border: '1px solid var(--color-border)',
              borderRadius: '6px',
              padding: '8px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s',
              boxShadow: '0 1px 2px 0 rgba(0, 0, 0, 0.05)'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'var(--color-surface)';
              e.currentTarget.style.transform = 'scale(1.05)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--color-background)';
              e.currentTarget.style.transform = 'scale(1)';
            }}
            title="Zoom in"
            disabled={zoom >= 3}
          >
            <ZoomIn className="w-4 h-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onZoomOut();
            }}
            style={{
              background: 'var(--color-background)',
              border: '1px solid var(--color-border)',
              borderRadius: '6px',
              padding: '8px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s',
              boxShadow: '0 1px 2px 0 rgba(0, 0, 0, 0.05)'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'var(--color-surface)';
              e.currentTarget.style.transform = 'scale(1.05)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--color-background)';
              e.currentTarget.style.transform = 'scale(1)';
            }}
            title="Zoom out"
            disabled={zoom <= 0.5}
          >
            <ZoomOut className="w-4 h-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onClose();
            }}
            style={{
              background: 'var(--color-background)',
              border: '1px solid var(--color-border)',
              borderRadius: '6px',
              padding: '8px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transition: 'all 0.2s',
              boxShadow: '0 1px 2px 0 rgba(0, 0, 0, 0.05)'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'var(--color-surface)';
              e.currentTarget.style.borderColor = 'var(--color-border)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--color-background)';
              e.currentTarget.style.borderColor = 'var(--color-border)';
            }}
            title="Close"
          >
            <X className="w-4 h-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
        </div>
        {/* Zoomed SVG Container */}
        <div
          style={{
            flex: 1,
            width: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            overflow: 'auto',
            transform: `scale(${zoom})`,
            transformOrigin: 'center center',
            transition: 'transform 0.2s ease',
            minHeight: 0
          }}
        >
          <div 
            dangerouslySetInnerHTML={{ __html: expandedMermaid.svg }}
            style={{ 
              width: '100%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center'
            }}
          />
        </div>
      </div>
    </div>
  );
}

