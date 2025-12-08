// apps/studio-core/src/components/EditorPane.tsx
import React from 'react';
import { SrujaMonacoEditor } from '@sruja/ui';

interface EditorPaneProps {
  dsl: string;
  onChange: (dsl: string) => void;
  onReady: (monaco: any, editor: any) => void;
  className?: string;
}

export function EditorPane({ dsl, onChange, onReady, className }: EditorPaneProps) {
  return (
    <div className={className || 'flex flex-1 flex-col'}>
      <SrujaMonacoEditor
        value={dsl}
        onChange={onChange}
        height="100%"
        onReady={onReady}
      />
    </div>
  );
}









