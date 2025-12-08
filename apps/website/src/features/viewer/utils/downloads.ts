import type { ArchitectureJSON } from '@sruja/viewer';

export const downloadHtml = (htmlPreview: string, archData: ArchitectureJSON | null) => {
  if (!htmlPreview) return;
  const archName = archData?.metadata?.name || 'architecture';
  const filename = `${archName.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.html`;
  const blob = new Blob([htmlPreview], { type: 'text/html' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

export const downloadMarkdown = (markdownPreview: string, archData: ArchitectureJSON | null) => {
  if (!markdownPreview) return;
  const archName = archData?.metadata?.name || 'architecture';
  const filename = `${archName.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.md`;
  const blob = new Blob([markdownPreview], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

export const downloadJson = (archData: ArchitectureJSON | null) => {
  if (!archData) return;
  const archName = archData?.metadata?.name || 'architecture';
  const filename = `${archName.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.json`;
  const jsonStr = JSON.stringify(archData, null, 2);
  const blob = new Blob([jsonStr], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

export const downloadPdf = (pdfPreviewUrl: string, archData: ArchitectureJSON | null) => {
  if (!pdfPreviewUrl) return;
  const archName = archData?.metadata?.name || 'architecture';
  const filename = `${archName.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.pdf`;
  const a = document.createElement('a');
  a.href = pdfPreviewUrl;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
};


