// Search dialog component for finding elements in the diagram

import React, { useState, useEffect, useRef, useMemo } from 'react';
import { Search, X, ArrowRight } from 'lucide-react';
import { ArchitectureJSON } from '@sruja/viewer';
import { Button } from '@sruja/ui';
import { validateSearchQuery } from '../utils/inputValidation';
import { useDebounce } from '../utils/useDebounce';

interface SearchResult {
  id: string;
  label: string;
  type: string;
  path: string; // e.g., "System > Container > Component"
}

interface SearchDialogProps {
  isOpen: boolean;
  archData: ArchitectureJSON | null;
  onSelect: (id: string) => void;
  onClose: () => void;
}

export const SearchDialog: React.FC<SearchDialogProps> = ({
  isOpen,
  archData,
  onSelect,
  onClose,
}) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Debounce search query to avoid expensive searches on every keystroke
  const debouncedQuery = useDebounce(query, 300);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      setQuery('');
      setResults([]);
      setSelectedIndex(0);
    }
  }, [isOpen]);

  // Memoize search results computation
  const searchResults = useMemo(() => {
    if (!debouncedQuery.trim() || !archData) {
      return [];
    }

    // Validate and sanitize search query
    const validation = validateSearchQuery(debouncedQuery);
    if (!validation.isValid) {
      return [];
    }

    const searchTerm = (validation.sanitized || debouncedQuery).toLowerCase();
    const found: SearchResult[] = [];

    const arch = archData.architecture;
    if (!arch) return;

    // Search persons
    if (arch.persons) {
      arch.persons.forEach((person) => {
        if (
          person.id.toLowerCase().includes(searchTerm) ||
          person.label?.toLowerCase().includes(searchTerm)
        ) {
          found.push({
            id: person.id,
            label: person.label || person.id,
            type: 'person',
            path: person.label || person.id,
          });
        }
      });
    }

    // Search systems and nested elements
    if (arch.systems) {
      arch.systems.forEach((system) => {
        if (
          system.id.toLowerCase().includes(searchTerm) ||
          system.label?.toLowerCase().includes(searchTerm)
        ) {
          found.push({
            id: system.id,
            label: system.label || system.id,
            type: 'system',
            path: system.label || system.id,
          });
        }

        // Search containers
        if (system.containers) {
          system.containers.forEach((container) => {
            if (
              container.id.toLowerCase().includes(searchTerm) ||
              container.label?.toLowerCase().includes(searchTerm)
            ) {
              found.push({
                id: `${system.id}.${container.id}`,
                label: container.label || container.id,
                type: 'container',
                path: `${system.label || system.id} > ${container.label || container.id}`,
              });
            }

            // Search components
            if (container.components) {
              container.components.forEach((component) => {
                if (
                  component.id.toLowerCase().includes(searchTerm) ||
                  component.label?.toLowerCase().includes(searchTerm)
                ) {
                  found.push({
                    id: `${system.id}.${container.id}.${component.id}`,
                    label: component.label || component.id,
                    type: 'component',
                    path: `${system.label || system.id} > ${container.label || container.id} > ${component.label || component.id}`,
                  });
                }
              });
            }
          });
        }

        // Search datastores
        if (system.datastores) {
          system.datastores.forEach((ds) => {
            if (
              ds.id.toLowerCase().includes(searchTerm) ||
              ds.label?.toLowerCase().includes(searchTerm)
            ) {
              found.push({
                id: `${system.id}.${ds.id}`,
                label: ds.label || ds.id,
                type: 'datastore',
                path: `${system.label || system.id} > ${ds.label || ds.id}`,
              });
            }
          });
        }

        // Search queues
        if (system.queues) {
          system.queues.forEach((queue) => {
            if (
              queue.id.toLowerCase().includes(searchTerm) ||
              queue.label?.toLowerCase().includes(searchTerm)
            ) {
              found.push({
                id: `${system.id}.${queue.id}`,
                label: queue.label || queue.id,
                type: 'queue',
                path: `${system.label || system.id} > ${queue.label || queue.id}`,
              });
            }
          });
        }
      });
    }

    // Search top-level containers
    if (arch.containers) {
      arch.containers.forEach((container) => {
        if (
          container.id.toLowerCase().includes(searchTerm) ||
          container.label?.toLowerCase().includes(searchTerm)
        ) {
          found.push({
            id: container.id,
            label: container.label || container.id,
            type: 'container',
            path: container.label || container.id,
          });
        }
      });
    }

    return found;
  }, [debouncedQuery, archData]);

  // Update results when search results change
  useEffect(() => {
    setResults(searchResults);
    setSelectedIndex(0);
  }, [searchResults]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && results.length > 0) {
      e.preventDefault();
      handleSelect(results[selectedIndex].id);
    }
  };

  const handleSelect = (id: string) => {
    onSelect(id);
    onClose();
  };

  const getTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      person: 'var(--color-primary-500)',
      system: 'var(--color-neutral-900)',
      container: 'var(--color-neutral-700)',
      component: 'var(--color-neutral-500)',
      datastore: 'var(--color-neutral-600)',
      queue: 'var(--color-neutral-700)',
    };
    return colors[type] || 'var(--color-neutral-500)';
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-start justify-center pt-[100px] z-50"
      onClick={onClose}
    >
      <div
        className="bg-[var(--color-background)] rounded-lg w-[600px] max-w-[90vw] shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div className="px-4 py-4 border-b border-[var(--color-border)] flex items-center gap-3">
          <Search size={20} className="text-[var(--color-text-tertiary)]" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search elements by name or ID..."
            className="flex-1 bg-transparent border-none outline-none text-[var(--color-text-primary)] text-base"
          />
          {query && (
            <Button variant="ghost" size="sm" onClick={() => setQuery('')} className="p-1">
              <X size={18} />
            </Button>
          )}
        </div>

        {/* Results */}
        {results.length > 0 && (
          <div className="max-h-[400px] overflow-y-auto">
            {results.map((result, index) => (
              <div
                key={result.id}
                onClick={() => handleSelect(result.id)}
                className={[
                  'px-4 py-3 cursor-pointer flex items-center gap-3 transition-colors',
                  index === selectedIndex ? 'bg-[var(--color-surface)]' : 'bg-[var(--color-background)]'
                ].join(' ')}
                onMouseEnter={() => setSelectedIndex(index)}
              >
                <div className="w-2 h-2 rounded-full" style={{ backgroundColor: getTypeColor(result.type) }} />
                <div className="flex-1">
                  <div className="text-sm font-semibold text-[var(--color-text-primary)] mb-0.5">
                    {result.label}
                  </div>
                  <div className="text-xs text-[var(--color-text-secondary)]">
                    {result.path}
                  </div>
                </div>
                <div className="text-[11px] text-[var(--color-text-tertiary)] capitalize px-2 py-0.5 rounded bg-[var(--color-surface)]">
                  {result.type}
                </div>
                {index === selectedIndex && (
                  <ArrowRight size={16} className="text-[var(--color-primary)]" />
                )}
              </div>
            ))}
          </div>
        )}

        {/* No Results */}
        {query && results.length === 0 && (
          <div className="p-8 text-center text-[var(--color-text-secondary)] text-sm">
            No elements found matching "{query}"
          </div>
        )}

        {/* Empty State */}
        {!query && (
          <div className="p-8 text-center text-[var(--color-text-tertiary)] text-xs">
            Start typing to search for elements...
            <div className="mt-2 text-[11px]">
              Use ↑↓ to navigate, Enter to select, Esc to close
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="px-4 py-2 border-t border-[var(--color-border)] text-[11px] text-[var(--color-text-tertiary)] flex justify-between">
          <span>{results.length} {results.length === 1 ? 'result' : 'results'}</span>
          <span>↑↓ Navigate • Enter Select • Esc Close</span>
        </div>
      </div>
    </div>
  );
};
