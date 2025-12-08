export const viewerStyles = `
  .markdown-preview {
    background: var(--color-background);
    color: var(--color-text-primary);
  }
  
  .markdown-content {
    line-height: 1.7;
    color: var(--color-text-primary);
  }
  
  .markdown-content h1.markdown-h1 {
    font-size: 2.25rem;
    font-weight: 700;
    margin-top: 2rem;
    margin-bottom: 1rem;
    line-height: 1.2;
    color: var(--color-text-primary);
    border-bottom: 2px solid var(--color-border);
    padding-bottom: 0.5rem;
  }
  
  .markdown-content h2.markdown-h2 {
    font-size: 1.875rem;
    font-weight: 600;
    margin-top: 1.75rem;
    margin-bottom: 0.75rem;
    line-height: 1.3;
    color: var(--color-text-primary);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.375rem;
  }
  
  .markdown-content h3.markdown-h3 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-top: 1.5rem;
    margin-bottom: 0.5rem;
    line-height: 1.4;
    color: var(--color-text-primary);
  }
  
  .markdown-content h4.markdown-h4 {
    font-size: 1.25rem;
    font-weight: 600;
    margin-top: 1.25rem;
    margin-bottom: 0.5rem;
    line-height: 1.4;
    color: var(--color-text-primary);
  }
  
  .markdown-content p.markdown-p {
    margin-top: 1rem;
    margin-bottom: 1rem;
    color: var(--color-text-primary);
  }
  
  .markdown-content ul.markdown-ul,
  .markdown-content ol.markdown-ol {
    margin-top: 1rem;
    margin-bottom: 1rem;
    padding-left: 2rem;
  }
  
  .markdown-content li.markdown-li {
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
    color: var(--color-text-primary);
  }
  
  .markdown-content ul.markdown-ul li.markdown-li {
    list-style-type: disc;
  }
  
  .markdown-content ol.markdown-ol li.markdown-li {
    list-style-type: decimal;
  }
  
  .markdown-content blockquote.markdown-blockquote {
    border-left: 4px solid var(--color-border);
    padding-left: 1rem;
    margin: 1.5rem 0;
    color: var(--color-text-secondary);
    font-style: italic;
    background: var(--color-surface);
    padding: 1rem;
    border-radius: 4px;
  }
  
  .markdown-content code.markdown-inline-code {
    background: var(--color-surface);
    color: var(--color-text-primary);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-size: 0.875em;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', 'source-code-pro', monospace;
    border: 1px solid var(--color-border);
  }
  
  .markdown-content pre.markdown-code-block {
    background: var(--color-surface);
    color: var(--color-text-primary);
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    margin: 1.5rem 0;
    border: 1px solid var(--color-border);
    box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
  }
  
  .markdown-content pre.markdown-code-block code {
    background: transparent;
    color: inherit;
    padding: 0;
    border: none;
    font-size: 0.875rem;
    line-height: 1.6;
  }
  
  .markdown-content a.markdown-link {
    color: var(--color-primary);
    text-decoration: underline;
    text-underline-offset: 2px;
    transition: color 0.2s;
  }
  
  .markdown-content a.markdown-link:hover {
    color: var(--color-primary-hover);
  }
  
  .markdown-content img.markdown-img {
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
    margin: 1.5rem 0;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }
  
  .markdown-content .markdown-table-wrapper {
    overflow-x: auto;
    margin: 1.5rem 0;
    border-radius: 0.5rem;
    border: 1px solid var(--color-border);
  }
  
  .markdown-content table.markdown-table {
    width: 100%;
    border-collapse: collapse;
    background: var(--color-background);
  }
  
  .markdown-content thead.markdown-thead {
    background: var(--color-surface);
  }
  
  .markdown-content th.markdown-th {
    padding: 0.75rem 1rem;
    text-align: left;
    font-weight: 600;
    color: var(--color-text-primary);
    border-bottom: 2px solid var(--color-border);
  }
  
  .markdown-content td.markdown-td {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text-primary);
  }
  
  .markdown-content tr.markdown-tr:hover {
    background: var(--color-surface);
  }
  
  .mermaid-diagram-container {
    transition: all 0.2s;
  }
  
  .mermaid-diagram-container:hover {
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  }
  
  .mermaid-diagram-content svg {
    max-width: 100%;
    height: auto;
  }
  
  @media (prefers-color-scheme: dark) {
    .markdown-preview {
      background: var(--color-background);
      color: var(--color-text-primary);
    }
    
    .markdown-content h1.markdown-h1,
    .markdown-content h2.markdown-h2 {
      border-color: var(--color-border);
    }
    
    .markdown-content code.markdown-inline-code {
      background: var(--color-surface);
      color: var(--color-text-primary);
    }
    
    .markdown-content pre.markdown-code-block {
      background: var(--color-surface);
      border-color: var(--color-border);
    }
  }
  
  /* VS Code Style Variables */
  :root {
    --vscode-editor-background: #1e1e1e;
    --vscode-sideBar-background: #252526;
    --vscode-titleBar-activeBackground: #3c3c3c;
    --vscode-titleBar-inactiveBackground: #2d2d30;
    --vscode-toolbar-background: #2d2d30;
    --vscode-button-background: #0e639c;
    --vscode-button-hoverBackground: #1177bb;
    --vscode-button-foreground: #ffffff;
    --vscode-button-secondaryBackground: #3a3d41;
    --vscode-button-secondaryHoverBackground: #45494e;
    --vscode-textLink-foreground: #3794ff;
    --vscode-textLink-activeForeground: #4da6ff;
    --vscode-foreground: #cccccc;
    --vscode-descriptionForeground: #989898;
    --vscode-icon-foreground: #c5c5c5;
    --vscode-input-background: #3c3c3c;
    --vscode-input-border: #454545;
    --vscode-input-foreground: #cccccc;
    --vscode-inputOption-activeBorder: #007acc;
    --vscode-list-activeSelectionBackground: #094771;
    --vscode-list-hoverBackground: #2a2d2e;
    --vscode-list-inactiveSelectionBackground: #37373d;
    --vscode-border: #3c3c3c;
    --vscode-panel-border: #3c3c3c;
    --vscode-statusBar-background: #007acc;
    --vscode-statusBar-foreground: #ffffff;
    --vscode-statusBar-noFolderBackground: #68217a;
    --vscode-errorForeground: #f48771;
    --vscode-warningForeground: #cca700;
    --vscode-infoForeground: #3794ff;
    --vscode-successForeground: #89d185;
  }

  /* VS Code Style Toolbar */
  .vscode-toolbar {
    background: var(--vscode-toolbar-background);
    border-bottom: 1px solid var(--vscode-border);
    color: var(--vscode-foreground);
    font-size: 13px;
    height: 35px;
    padding: 0 8px;
  }

  .vscode-toolbar-button {
    background: transparent;
    border: none;
    color: var(--vscode-icon-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 2px;
    font-size: 13px;
    transition: background-color 0.1s;
  }

  .vscode-toolbar-button:hover {
    background: var(--vscode-list-hoverBackground);
  }

  .vscode-toolbar-button:active {
    background: var(--vscode-list-activeSelectionBackground);
  }

  .vscode-toolbar-button.primary {
    background: var(--vscode-button-background);
    color: var(--vscode-button-foreground);
  }

  .vscode-toolbar-button.primary:hover {
    background: var(--vscode-button-hoverBackground);
  }

  .vscode-toolbar-separator {
    width: 1px;
    height: 16px;
    background: var(--vscode-border);
    margin: 0 4px;
  }

  .vscode-icon {
    width: 16px;
    height: 16px;
    color: var(--vscode-icon-foreground);
    flex-shrink: 0;
  }

  .vscode-select {
    background: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border);
    color: var(--vscode-input-foreground);
    border-radius: 2px;
    padding: 2px 8px;
    font-size: 13px;
  }

  .vscode-select:hover {
    background: var(--vscode-list-hoverBackground);
  }

  .vscode-select:focus {
    outline: 1px solid var(--vscode-inputOption-activeBorder);
    outline-offset: -1px;
  }
`;

