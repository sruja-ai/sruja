// apps/studio-core/src/components/MarkdownPreviewModal.tsx
import React from 'react';
import { X, Copy } from 'lucide-react';
import { MarkdownPreview } from './MarkdownPreview';

interface MarkdownPreviewModalProps {
  isOpen: boolean;
  content: string;
  mode: 'preview' | 'raw';
  onClose: () => void;
  onModeChange: (mode: 'preview' | 'raw') => void;
  onCopy: () => void;
}

export function MarkdownPreviewModal({
  isOpen,
  content,
  mode,
  onClose,
  onModeChange,
  onCopy,
}: MarkdownPreviewModalProps) {
  if (!isOpen) return null;

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
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              cursor: 'pointer',
              padding: '4px',
              color: 'var(--color-text-secondary)',
            }}
          >
            <X size={20} />
          </button>
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









