// apps/viewer-core/app/components/ExportDialog.tsx
import React, { useState, useEffect } from 'react';
import { Download, X, FileImage, FileCode, FileText, FileJson } from 'lucide-react';
import { Button } from '@sruja/ui';

export type ExportFormat = 'png' | 'svg' | 'pdf' | 'json' | 'markdown' | 'html';

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
      case 'html':
        return <FileCode size={18} />;
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onCancel}>
      <div className="bg-[var(--color-background)] rounded-lg w-[480px] max-w-[90vw] max-h-[90vh] overflow-auto shadow-xl" onClick={(e) => e.stopPropagation()}>
        <div className="px-6 py-5 border-b border-[var(--color-border)] flex items-center justify-between">
          <h2 className="m-0 text-lg font-semibold text-[var(--color-text-primary)]">Export Diagram</h2>
          <Button variant="ghost" size="sm" onClick={onCancel} className="p-1">
            <X size={20} />
          </Button>
        </div>

        <div className="p-6">
          <div className="mb-6">
            <label className="block mb-2 text-sm font-medium text-[var(--color-text-secondary)]">Format</label>
            <div className="flex gap-2 flex-wrap">
              {(['png', 'svg', 'pdf', 'json', 'markdown', 'html'] as ExportFormat[]).map((fmt) => (
                <button
                  key={fmt}
                  onClick={() => setFormat(fmt)}
                  className={[
                    'flex-1 min-w-[100px] px-4 py-2 rounded-md border transition-all flex items-center justify-center gap-2 text-sm',
                    format === fmt
                      ? 'border-[var(--color-primary)] bg-[var(--color-surface)] text-[var(--color-primary)] font-semibold'
                      : 'border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text-secondary)]'
                  ].join(' ')}
                >
                  {getFormatIcon(fmt)}
                  {fmt.toUpperCase()}
                </button>
              ))}
            </div>
          </div>

          {(format === 'png' || format === 'svg') && (
            <div className="mb-6">
              <label className="block mb-2 text-sm font-medium text-[var(--color-text-secondary)]">Quality</label>
              <div className="flex gap-2">
                {(['standard', 'high', '4k'] as const).map((q) => (
                  <button
                    key={q}
                    onClick={() => {
                      setQuality(q);
                      setScale(q === 'standard' ? 1 : q === 'high' ? 2 : 4);
                    }}
                    className={[
                      'flex-1 px-4 py-2 rounded-md border text-sm capitalize transition-all',
                      quality === q
                        ? 'border-[var(--color-primary)] bg-[var(--color-surface)] text-[var(--color-primary)] font-semibold'
                        : 'border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text-secondary)]'
                    ].join(' ')}
                  >
                    {q === '4k' ? '4K' : q}
                  </button>
                ))}
              </div>
              <div className="mt-2 text-xs text-[var(--color-text-tertiary)]">Scale: {scale}x ({scale * 100}% resolution)</div>
            </div>
          )}

          <div className="mb-6">
            <div className="mb-3">
              <label className="inline-flex items-center gap-2 cursor-pointer text-sm text-[var(--color-text-secondary)]">
                <input type="checkbox" checked={includeMetadata} onChange={(e) => setIncludeMetadata(e.target.checked)} />
                Include metadata (author, date, version)
              </label>
            </div>
            <div>
              <label className="inline-flex items-center gap-2 cursor-pointer text-sm text-[var(--color-text-secondary)]">
                <input type="checkbox" checked={includeTimestamp} onChange={(e) => setIncludeTimestamp(e.target.checked)} />
                Include timestamp in filename
              </label>
            </div>
          </div>

          <div className="mb-6">
            <label className="block mb-2 text-sm font-medium text-[var(--color-text-secondary)]">Filename</label>
            <input
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              className="w-full px-3 py-2 rounded-md border border-[var(--color-border)] text-[var(--color-text-primary)] bg-[var(--color-background)] focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[var(--color-primary)] text-sm"
              placeholder="architecture"
            />
          </div>

          <div>
            <label className="block mb-2 text-sm font-medium text-[var(--color-text-secondary)]">Quick Presets</label>
            <div className="flex gap-2 flex-wrap">
              {(['print', 'presentation', 'documentation', 'web'] as const).map((preset) => (
                <button
                  key={preset}
                  onClick={() => applyPreset(preset)}
                  className="px-4 py-2 rounded-md border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text-secondary)] text-xs font-medium"
                >
                  {preset.charAt(0).toUpperCase() + preset.slice(1)}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="px-6 py-4 border-t border-[var(--color-border)] flex justify-end gap-3">
          <Button variant="ghost" onClick={onCancel}>Cancel</Button>
          <Button variant="primary" onClick={handleExport}>
            <Download size={16} />
            Export
          </Button>
        </div>
      </div>
    </div>
  );
};
