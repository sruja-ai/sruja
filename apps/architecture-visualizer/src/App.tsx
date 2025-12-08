import { useCallback, useState } from 'react';
import { Upload, FileJson } from 'lucide-react';
import { ArchitectureCanvas } from './components/Canvas';
import { NavigationPanel, DetailsPanel } from './components/Panels';
import { Breadcrumb, ExamplesDropdown, ThemeToggle } from './components/shared';
import { useArchitectureStore, useSelectionStore } from './stores';
import './App.css';

// Sample JSON for demo
const DEMO_JSON = {
  metadata: { name: 'Demo Architecture', version: '1.0.0', generated: new Date().toISOString() },
  architecture: {
    systems: [
      { id: 'WebApp', label: 'Web Application', description: 'User-facing web app', containers: [] },
      { id: 'API', label: 'API Service', description: 'Backend REST API', containers: [] },
      { id: 'Database', label: 'Database', description: 'PostgreSQL database' },
    ],
    persons: [
      { id: 'User', label: 'User', description: 'End user of the system' },
    ],
    relations: [
      { from: 'User', to: 'WebApp', label: 'Uses' },
      { from: 'WebApp', to: 'API', label: 'Calls' },
      { from: 'API', to: 'Database', label: 'Reads/Writes' },
    ],
  },
  navigation: { levels: ['L1', 'L2', 'L3'] },
};

export default function App() {
  const data = useArchitectureStore((s) => s.data);
  const loadFromJSON = useArchitectureStore((s) => s.loadFromJSON);
  const selectedNodeId = useSelectionStore((s) => s.selectedNodeId);
  const [isLoadingFile, setIsLoadingFile] = useState(false);

  // Handle file drop
  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const file = e.dataTransfer.files[0];
      if (file && file.type === 'application/json') {
        setIsLoadingFile(true);
        const reader = new FileReader();
        reader.onload = (event) => {
          try {
            const json = JSON.parse(event.target?.result as string);
            loadFromJSON(json);
          } catch (err) {
            console.error('Invalid JSON file');
          } finally {
            setIsLoadingFile(false);
          }
        };
        reader.readAsText(file);
      }
    },
    [loadFromJSON]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  // Handle file input
  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) {
        setIsLoadingFile(true);
        const reader = new FileReader();
        reader.onload = (event) => {
          try {
            const json = JSON.parse(event.target?.result as string);
            loadFromJSON(json);
          } catch (err) {
            console.error('Invalid JSON file');
          } finally {
            setIsLoadingFile(false);
          }
        };
        reader.readAsText(file);
      }
    },
    [loadFromJSON]
  );

  // Load demo data
  const loadDemo = useCallback(() => {
    loadFromJSON(DEMO_JSON);
  }, [loadFromJSON]);

  return (
    <div className="app" onDrop={handleDrop} onDragOver={handleDragOver}>
      {/* Header */}
      <header className="app-header">
        <div className="header-left">
          <h1 className="app-title">
            <FileJson size={20} />
            Architecture Visualizer
          </h1>
        </div>
        <div className="header-center">
          {data && <Breadcrumb />}
        </div>
        <div className="header-right">
          <ThemeToggle />
          <ExamplesDropdown />
          <label className="upload-btn">
            <Upload size={16} />
            Load JSON
            <input
              type="file"
              accept=".json"
              onChange={handleFileSelect}
              style={{ display: 'none' }}
            />
          </label>
        </div>
      </header>

      {/* Main Content */}
      <main className="app-main">
        <NavigationPanel />

        <div className="canvas-container">
          {!data && !isLoadingFile && (
            <div className="drop-zone">
              <Upload size={48} strokeWidth={1.5} />
              <h2>Drop JSON file here</h2>
              <p>Or use the "Load JSON" button, or try the demo</p>
              <button className="demo-btn large" onClick={loadDemo}>
                Load Demo Architecture
              </button>
            </div>
          )}
          {isLoadingFile && (
            <div className="loading">Loading...</div>
          )}
          {data && <ArchitectureCanvas />}
        </div>

        {selectedNodeId && <DetailsPanel />}
      </main>
    </div>
  );
}
