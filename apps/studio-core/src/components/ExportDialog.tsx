// Professional export dialog component

import React, { useState, useEffect } from 'react';
import { Download, X, FileImage, FileCode, FileText, FileJson } from 'lucide-react';
import { Button } from '@sruja/ui';

export type ExportFormat = 'png' | 'svg' | 'pdf' | 'json' | 'markdown';

export interface ExportOptions {
  format: ExportFormat;
  quality: 'standard' | 'high' | '4k';
  scale: number;
  includeMetadata: boolean;
  includeTimestamp: boolean;
  filename: string;
}

interface ExportDialogProps {
  isOpen: boolean;
  defaultFilename?: string;
  onExport: (options: ExportOptions) => void;
  onCancel: () => void;
}

const EXPORT_PRESETS = {
  print: {
    format: 'png' as ExportFormat,
    quality: '4k' as const,
    scale: 4,
    includeMetadata: true,
    includeTimestamp: false,
  },
  presentation: {
    format: 'png' as ExportFormat,
    quality: 'high' as const,
    scale: 2,
    includeMetadata: false,
    includeTimestamp: false,
  },
  documentation: {
    format: 'svg' as ExportFormat,
    quality: 'standard' as const,
    scale: 1,
    includeMetadata: true,
    includeTimestamp: true,
  },
  web: {
    format: 'png' as ExportFormat,
    quality: 'standard' as const,
    scale: 2,
    includeMetadata: false,
    includeTimestamp: false,
  },
};

export const ExportDialog: React.FC<ExportDialogProps> = ({
  isOpen,
  defaultFilename = 'architecture',
  onExport,
  onCancel,
}) => {
  const [format, setFormat] = useState<ExportFormat>('png');
  const [quality, setQuality] = useState<'standard' | 'high' | '4k'>('high');
  const [scale, setScale] = useState(2);
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeTimestamp, setIncludeTimestamp] = useState(false);
  const [filename, setFilename] = useState(defaultFilename);

  useEffect(() => {
    if (isOpen) {
      // Generate default filename with timestamp if needed
      const timestamp = new Date().toISOString().split('T')[0];
      setFilename(`${defaultFilename}_${timestamp}`);
    }
  }, [isOpen, defaultFilename]);

  const applyPreset = (preset: keyof typeof EXPORT_PRESETS) => {
    const presetOptions = EXPORT_PRESETS[preset];
    setFormat(presetOptions.format);
    setQuality(presetOptions.quality);
    setScale(presetOptions.scale);
    setIncludeMetadata(presetOptions.includeMetadata);
    setIncludeTimestamp(presetOptions.includeTimestamp);
  };

  const handleExport = () => {
    onExport({
      format,
      quality,
      scale,
      includeMetadata,
      includeTimestamp,
      filename,
    });
  };

  const getFormatIcon = (fmt: ExportFormat) => {
    switch (fmt) {
      case 'png':
        return <FileImage size={18} />;
      case 'svg':
        return <FileCode size={18} />;
      case 'pdf':
        return <FileText size={18} />;
      case 'json':
        return <FileJson size={18} />;
      case 'markdown':
        return <FileText size={18} />;
    }
  };

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'var(--overlay-scrim)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onCancel}
    >
      <div
        style={{
          backgroundColor: 'var(--color-background)',
          borderRadius: '8px',
          width: '480px',
          maxWidth: '90vw',
          maxHeight: '90vh',
          overflow: 'auto',
          boxShadow: 'var(--shadow-xl)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          style={{
            padding: '20px 24px',
            borderBottom: '1px solid var(--color-border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <h2 style={{ margin: 0, fontSize: '18px', fontWeight: 600, color: 'var(--color-text-primary)' }}>
            Export Diagram
          </h2>
          <Button variant="ghost" size="sm" onClick={onCancel} style={{ padding: '4px' }}>
            <X size={20} />
          </Button>
        </div>

        {/* Content */}
        <div style={{ padding: '24px' }}>
          {/* Format Selection */}
          <div style={{ marginBottom: '24px' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '8px',
                fontSize: '14px',
                fontWeight: 500,
                color: 'var(--color-text-secondary)',
              }}
            >
              Format
            </label>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              {(['png', 'svg', 'pdf', 'json'] as ExportFormat[]).map((fmt) => (
                <button
                  key={fmt}
                  onClick={() => setFormat(fmt)}
                  style={{
                    flex: 1,
                    minWidth: '100px',
                    padding: '10px 16px',
                    borderRadius: '6px',
                    border: `2px solid ${format === fmt ? 'var(--color-primary)' : 'var(--color-border)'}`,
                    backgroundColor: format === fmt ? 'var(--color-surface)' : 'var(--color-background)',
                    color: format === fmt ? 'var(--color-primary)' : 'var(--color-text-secondary)',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: '6px',
                    fontSize: '14px',
                    fontWeight: format === fmt ? 600 : 500,
                    transition: 'all 0.2s',
                  }}
                >
                  {getFormatIcon(fmt)}
                  {fmt.toUpperCase()}
                </button>
              ))}
            </div>
          </div>

          {/* Quality (for PNG) */}
          {format === 'png' && (
            <div style={{ marginBottom: '24px' }}>
              <label
                style={{
                  display: 'block',
                  marginBottom: '8px',
                  fontSize: '14px',
                  fontWeight: 500,
                  color: '#475569',
                }}
              >
                Quality
              </label>
              <div style={{ display: 'flex', gap: '8px' }}>
                {(['standard', 'high', '4k'] as const).map((q) => (
                  <button
                    key={q}
                    onClick={() => {
                      setQuality(q);
                      setScale(q === 'standard' ? 1 : q === 'high' ? 2 : 4);
                    }}
                    style={{
                      flex: 1,
                      padding: '10px 16px',
                      borderRadius: '6px',
                    border: `2px solid ${quality === q ? 'var(--color-primary)' : 'var(--color-border)'}`,
                    backgroundColor: quality === q ? 'var(--color-surface)' : 'var(--color-background)',
                    color: quality === q ? 'var(--color-primary)' : 'var(--color-text-secondary)',
                      cursor: 'pointer',
                      fontSize: '14px',
                      fontWeight: quality === q ? 600 : 500,
                      textTransform: 'capitalize',
                    }}
                  >
                    {q === '4k' ? '4K' : q}
                  </button>
                ))}
              </div>
              <div style={{ marginTop: '8px', fontSize: '12px', color: 'var(--color-text-tertiary)' }}>
                Scale: {scale}x ({scale * 100}% resolution)
              </div>
            </div>
          )}

          {/* Options */}
          <div style={{ marginBottom: '24px' }}>
            <div style={{ marginBottom: '12px' }}>
              <label
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  cursor: 'pointer',
                  fontSize: '14px',
                  color: 'var(--color-text-secondary)',
                }}
              >
                <input
                  type="checkbox"
                  checked={includeMetadata}
                  onChange={(e) => setIncludeMetadata(e.target.checked)}
                  style={{ cursor: 'pointer' }}
                />
                Include metadata (author, date, version)
              </label>
            </div>
            <div>
              <label
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  cursor: 'pointer',
                  fontSize: '14px',
                  color: '#475569',
                }}
              >
                <input
                  type="checkbox"
                  checked={includeTimestamp}
                  onChange={(e) => setIncludeTimestamp(e.target.checked)}
                  style={{ cursor: 'pointer' }}
                />
                Include timestamp in filename
              </label>
            </div>
          </div>

          {/* Filename */}
          <div style={{ marginBottom: '24px' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '8px',
                fontSize: '14px',
                fontWeight: 500,
                color: 'var(--color-text-secondary)',
              }}
            >
              Filename
            </label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              style={{
                width: '100%',
                padding: '8px 12px',
                borderRadius: '6px',
                border: '1px solid var(--color-border)',
                fontSize: '14px',
                color: 'var(--color-text-primary)',
                outline: 'none',
              }}
              placeholder="architecture"
            />
          </div>

          {/* Presets */}
          <div>
            <label
              style={{
                display: 'block',
                marginBottom: '8px',
                fontSize: '14px',
                fontWeight: 500,
                color: 'var(--color-text-secondary)',
              }}
            >
              Quick Presets
            </label>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              <button
                onClick={() => applyPreset('print')}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid var(--color-border)',
                  backgroundColor: 'var(--color-background)',
                  color: 'var(--color-text-secondary)',
                  cursor: 'pointer',
                  fontSize: '13px',
                  fontWeight: 500,
                }}
              >
                Print
              </button>
              <button
                onClick={() => applyPreset('presentation')}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid #e2e8f0',
                  backgroundColor: '#fff',
                  color: '#475569',
                  cursor: 'pointer',
                  fontSize: '13px',
                  fontWeight: 500,
                }}
              >
                Presentation
              </button>
              <button
                onClick={() => applyPreset('documentation')}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid #e2e8f0',
                  backgroundColor: '#fff',
                  color: '#475569',
                  cursor: 'pointer',
                  fontSize: '13px',
                  fontWeight: 500,
                }}
              >
                Documentation
              </button>
              <button
                onClick={() => applyPreset('web')}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid #e2e8f0',
                  backgroundColor: '#fff',
                  color: '#475569',
                  cursor: 'pointer',
                  fontSize: '13px',
                  fontWeight: 500,
                }}
              >
                Web
              </button>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div
          style={{
            padding: '16px 24px',
            borderTop: '1px solid var(--color-border)',
            display: 'flex',
            justifyContent: 'flex-end',
            gap: '12px',
          }}
        >
          <Button variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleExport}>
            <Download size={16} />
            Export
          </Button>
        </div>
      </div>
    </div>
  );
};
