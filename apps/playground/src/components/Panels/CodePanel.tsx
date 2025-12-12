import { useState } from 'react';
import { DSLPanel } from './DSLPanel';
import { JSONPanel } from './JSONPanel';
import { MarkdownPanel } from './MarkdownPanel';
import { Code, FileJson, FileText } from 'lucide-react';
import './CodePanel.css';

type CodeTab = 'dsl' | 'json' | 'markdown';

export function CodePanel() {
    const [activeTab, setActiveTab] = useState<CodeTab>('dsl');

    return (
        <div className="code-panel-container">
            <div className="code-tabs">
                <button
                    className={`code-tab ${activeTab === 'dsl' ? 'active' : ''}`}
                    onClick={() => setActiveTab('dsl')}
                >
                    <Code size={16} />
                    <span>Sruja DSL</span>
                </button>
                <button
                    className={`code-tab ${activeTab === 'json' ? 'active' : ''}`}
                    onClick={() => setActiveTab('json')}
                >
                    <FileJson size={16} />
                    <span>JSON</span>
                </button>
                <button
                    className={`code-tab ${activeTab === 'markdown' ? 'active' : ''}`}
                    onClick={() => setActiveTab('markdown')}
                >
                    <FileText size={16} />
                    <span>Markdown</span>
                </button>
            </div>
            <div className="code-content">
                {activeTab === 'dsl' && <DSLPanel />}
                {activeTab === 'json' && <JSONPanel />}
                {activeTab === 'markdown' && <MarkdownPanel />}
            </div>
        </div>
    );
}
