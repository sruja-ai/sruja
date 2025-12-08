// apps/viewer-core/app/components/MarkdownPreviewModal.tsx
import React from 'react';
import { X, Copy, ExternalLink } from 'lucide-react';
import { MarkdownPreview } from './MarkdownPreview';
import { Button } from '@sruja/ui';
import LZString from 'lz-string';

interface MarkdownPreviewModalProps {
  isOpen: boolean;
  content: string;
  mode: 'preview' | 'raw';
  onClose: () => void;
  onModeChange: (mode: 'preview' | 'raw') => void;
  onCopy: () => void;
  dsl?: string; // Optional DSL code to generate "View in Viewer" link
}

export function MarkdownPreviewModal({
  isOpen,
  content,
  mode,
  onClose,
  onModeChange,
  onCopy,
  dsl,
}: MarkdownPreviewModalProps) {
  if (!isOpen) return null;

  const handleViewInViewer = () => {
    if (!dsl) return;
    
    // Generate shareable URL with compressed DSL
    const base = typeof window !== 'undefined' ? window.location.origin : '';
    const viewerPath = '/viewer';
    const compressed = LZString.compressToBase64(dsl);
    const url = `${base}${viewerPath}?code=${encodeURIComponent(compressed)}`;
    
    // Open in new tab
    window.open(url, '_blank');
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0,0,0,0.5)',
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      zIndex: 1000,
    }}>
      <div style={{
        backgroundColor: 'var(--color-background)',
        borderRadius: '8px',
        width: '800px',
        maxWidth: '90vw',
        height: '80vh',
        display: 'flex',
        flexDirection: 'column',
        boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
      }}>
        <div style={{
          padding: '16px 24px',
          borderBottom: '1px solid var(--color-border)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}>
          <h3 style={{ margin: 0, fontSize: '1.125rem', fontWeight: 600, color: 'var(--color-text-primary)' }}>Markdown Preview</h3>
          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
            style={{ padding: '4px' }}
          >
            <X size={20} />
          </Button>
        </div>

        <div style={{ flex: 1, padding: '0', overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <div style={{
            padding: '12px',
            backgroundColor: 'var(--color-surface)',
            borderBottom: '1px solid var(--color-border)',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center'
          }}>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={() => onModeChange('preview')}
                style={{
                  padding: '6px 12px',
                  borderRadius: '4px',
                  border: '1px solid var(--color-border)',
                  backgroundColor: mode === 'preview' ? 'var(--color-info-500)' : 'var(--color-background)',
                  color: mode === 'preview' ? 'var(--color-background)' : 'var(--color-text-primary)',
                  fontSize: '0.875rem',
                  cursor: 'pointer',
                  fontWeight: mode === 'preview' ? 600 : 400,
                }}
              >
                Preview
              </button>
              <button
                onClick={() => onModeChange('raw')}
                style={{
                  padding: '6px 12px',
                  borderRadius: '4px',
                  border: '1px solid var(--color-border)',
                  backgroundColor: mode === 'raw' ? 'var(--color-info-500)' : 'var(--color-background)',
                  color: mode === 'raw' ? 'var(--color-background)' : 'var(--color-text-primary)',
                  fontSize: '0.875rem',
                  cursor: 'pointer',
                  fontWeight: mode === 'raw' ? 600 : 400,
                }}
              >
                Raw
              </button>
            </div>
            <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
              {dsl && (
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={handleViewInViewer}
                  style={{ fontSize: '0.875rem' }}
                >
                  <ExternalLink size={16} style={{ marginRight: '4px' }} />
                  View in Viewer
                </Button>
              )}
              <button
                onClick={onCopy}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '6px 12px',
                  borderRadius: '4px',
                  border: '1px solid var(--color-border)',
                  backgroundColor: 'var(--color-background)',
                  color: 'var(--color-text-primary)',
                  fontSize: '0.875rem',
                  cursor: 'pointer',
                }}
              >
                <Copy size={16} />
                Copy
              </button>
            </div>
          </div>
          {mode === 'raw' ? (
            <textarea
              readOnly
              value={content}
              style={{
                flex: 1,
                width: '100%',
                padding: '16px',
                border: 'none',
                resize: 'none',
                fontFamily: 'monospace',
                fontSize: '0.875rem',
                lineHeight: '1.5',
                outline: 'none',
              }}
            />
          ) : (
            <MarkdownPreview content={content} />
          )}
        </div>
      </div>
    </div>
  );
}








