import { useState } from 'react';
import { ChevronDown, FileCode } from 'lucide-react';
import { EXAMPLES, fetchExampleJson } from '../../examples';
import { useArchitectureStore } from '../../stores';
import type { ArchitectureJSON } from '../../types';
import './ExamplesDropdown.css';

export function ExamplesDropdown() {
    const [isOpen, setIsOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const loadFromJSON = useArchitectureStore((s) => s.loadFromJSON);

    const handleExampleSelect = async (filename: string, name: string) => {
        setIsOpen(false);
        setLoading(true);
        setError(null);

        try {
            // Load pre-exported JSON directly (no WASM needed)
            const json = await fetchExampleJson(filename);
            loadFromJSON(json as ArchitectureJSON);
            console.log(`Loaded example: ${name}`);
        } catch (err) {
            const message = err instanceof Error ? err.message : 'Unknown error';
            setError(`Failed to load ${name}: ${message}`);
            console.error('Example load error:', err);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="examples-dropdown">
            <button
                className="examples-trigger"
                onClick={() => setIsOpen(!isOpen)}
                disabled={loading}
            >
                <FileCode size={16} />
                <span>{loading ? 'Loading...' : 'Examples'}</span>
                <ChevronDown size={14} className={isOpen ? 'rotated' : ''} />
            </button>

            {isOpen && (
                <div className="examples-menu">
                    {EXAMPLES.map((example) => (
                        <button
                            key={example.file}
                            className="example-item"
                            onClick={() => handleExampleSelect(example.file, example.name)}
                        >
                            <span className="example-name">{example.name}</span>
                            <span className="example-desc">{example.description}</span>
                        </button>
                    ))}
                </div>
            )}

            {error && <div className="examples-error">{error}</div>}
        </div>
    );
}
