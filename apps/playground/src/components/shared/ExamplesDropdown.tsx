import { useState, useEffect } from 'react';
import { FileCode } from 'lucide-react';
import { getAllExamples, fetchExampleDsl } from '../../examples';
import { convertDslToJson } from '../../wasm';
import { convertJsonToDsl } from '../../utils/jsonToDsl';
import { useArchitectureStore } from '../../stores';
import type { ArchitectureJSON } from '../../types';
import type { Example } from '@sruja/shared';
import './ExamplesDropdown.css';

export function ExamplesDropdown() {
    const [isOpen, setIsOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [examples, setExamples] = useState<Array<Example & { isDsl: boolean }>>([]);
    const loadFromDSL = useArchitectureStore((s) => s.loadFromDSL);

    // Load examples from shared service
    useEffect(() => {
        getAllExamples()
            .then((all) => setExamples(all.filter(e => e.isDsl)))
            .catch(err => {
                console.error('Failed to load examples:', err);
                setError('Failed to load examples');
            });
    }, []);

    const handleExampleSelect = async (example: Example & { isDsl: boolean }) => {
        setIsOpen(false);
        setLoading(true);
        setError(null);

        try {
            if (example.isDsl) {
                const dsl = await fetchExampleDsl(example.file);
                const converted = await convertDslToJson(dsl);
                if (!converted) {
                    throw new Error('Failed to parse DSL');
                }
                loadFromDSL(converted as ArchitectureJSON, dsl, example.file);
            } else {
                const jsonText = await fetchExampleDsl(example.file);
                const parsed = JSON.parse(jsonText) as ArchitectureJSON;
                const dsl = convertJsonToDsl(parsed);
                loadFromDSL(parsed, dsl, example.file);
            }
            console.log(`Loaded example: ${example.name}`);
        } catch (err) {
            const message = err instanceof Error ? err.message : 'Unknown error';
            setError(`Failed to load ${example.name}: ${message}`);
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
                title={loading ? 'Loading examples...' : 'Examples'}
                aria-label={loading ? 'Loading examples...' : 'Examples'}
            >
                <FileCode size={16} />
            </button>

            {isOpen && (
                <div className="examples-menu">
                    {examples.length === 0 && !error && (
                        <div className="examples-loading">Loading examples...</div>
                    )}
                    {Object.entries(
                        examples.reduce((acc, example) => {
                            if (!acc[example.category]) {
                                acc[example.category] = [];
                            }
                            acc[example.category].push(example);
                            return acc;
                        }, {} as Record<string, Array<Example & { isDsl: boolean }>>)
                    ).map(([category, categoryExamples]) => (
                        <div key={category}>
                            <div className="example-category">{category}</div>
                            {categoryExamples.sort((a, b) => a.order - b.order).map((example) => (
                                <button
                                    key={example.file}
                                    className="example-item"
                                    onClick={() => handleExampleSelect(example)}
                                >
                                    <div className="example-item-content">
                                        <span className="example-name">{example.name}</span>
                                        <span className="example-desc">{example.description}</span>
                                    </div>
                                    {example.isDsl && <span className="example-badge">DSL</span>}
                                </button>
                            ))}
                        </div>
                    ))}
                </div>
            )}

            {error && <div className="examples-error">{error}</div>}
        </div>
    );
}
