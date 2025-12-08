// Command palette component (Cmd+K) for quick actions

import React, { useState, useEffect, useRef } from 'react';
import {
  Search,
  X,
  ArrowRight,
} from 'lucide-react';
import { Button } from '@sruja/ui';

export interface Command {
  id: string;
  label: string;
  description?: string;
  icon: React.ReactNode;
  action: () => void;
  keywords?: string[];
  category?: string;
}

interface CommandPaletteProps {
  isOpen: boolean;
  commands: Command[];
  onClose: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  commands,
  onClose,
}) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Filter commands based on query
  const filteredCommands = commands.filter((cmd) => {
    if (!query.trim()) return true;
    const searchTerm = query.toLowerCase();
    return (
      cmd.label.toLowerCase().includes(searchTerm) ||
      cmd.description?.toLowerCase().includes(searchTerm) ||
      cmd.keywords?.some((kw) => kw.toLowerCase().includes(searchTerm)) ||
      cmd.category?.toLowerCase().includes(searchTerm)
    );
  });

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      setQuery('');
      setSelectedIndex(0);
    }
  }, [isOpen]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, filteredCommands.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && filteredCommands.length > 0) {
      e.preventDefault();
      filteredCommands[selectedIndex].action();
      onClose();
    }
  };

  const handleSelect = (command: Command) => {
    command.action();
    onClose();
  };

  // Group commands by category
  const groupedCommands = filteredCommands.reduce((acc, cmd) => {
    const category = cmd.category || 'Other';
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(cmd);
    return acc;
  }, {} as Record<string, Command[]>);

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
        alignItems: 'flex-start',
        justifyContent: 'center',
        paddingTop: '100px',
        zIndex: 1001,
      }}
      onClick={onClose}
    >
      <div
        style={{
        backgroundColor: 'var(--color-background)',
          borderRadius: '8px',
          width: '640px',
          maxWidth: '90vw',
        boxShadow: 'var(--shadow-xl)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div
          style={{
            padding: '16px',
            borderBottom: filteredCommands.length > 0 ? '1px solid var(--color-border)' : 'none',
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
          }}
        >
          <Search size={20} style={{ color: 'var(--color-text-tertiary)' }} />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search..."
            style={{
              flex: 1,
              border: 'none',
              outline: 'none',
              fontSize: '16px',
              color: 'var(--color-text-primary)',
            }}
          />
          {query && (
            <Button variant="ghost" size="sm" onClick={() => setQuery('')} style={{ padding: '4px' }}>
              <X size={18} />
            </Button>
          )}
        </div>

        {/* Commands List */}
        {filteredCommands.length > 0 && (
          <div
            style={{
              maxHeight: '400px',
              overflowY: 'auto',
            }}
          >
            {Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category}>
                {Object.keys(groupedCommands).length > 1 && (
                  <div
                    style={{
                      padding: '8px 16px',
                      fontSize: '11px',
                      fontWeight: 600,
                      color: 'var(--color-text-tertiary)',
                      textTransform: 'uppercase',
                      letterSpacing: '0.5px',
                      backgroundColor: 'var(--color-surface)',
                    }}
                  >
                    {category}
                  </div>
                )}
                {cmds.map((cmd) => {
                  const globalIndex = filteredCommands.indexOf(cmd);
                  return (
                    <div
                      key={cmd.id}
                      onClick={() => handleSelect(cmd)}
                      style={{
                        padding: '12px 16px',
                        cursor: 'pointer',
                        backgroundColor: globalIndex === selectedIndex ? 'var(--color-surface)' : 'var(--color-background)',
                        borderLeft: `3px solid ${globalIndex === selectedIndex ? 'var(--color-primary)' : 'transparent'}`,
                        display: 'flex',
                        alignItems: 'center',
                        gap: '12px',
                        transition: 'background-color 0.1s',
                      }}
                      onMouseEnter={() => setSelectedIndex(globalIndex)}
                    >
                      <div style={{ color: globalIndex === selectedIndex ? 'var(--color-primary)' : 'var(--color-text-tertiary)' }}>
                        {cmd.icon}
                      </div>
                      <div style={{ flex: 1 }}>
                        <div
                          style={{
                            fontSize: '14px',
                            fontWeight: 600,
                            color: 'var(--color-text-primary)',
                            marginBottom: cmd.description ? '2px' : 0,
                          }}
                        >
                          {cmd.label}
                        </div>
                        {cmd.description && (
                          <div
                            style={{
                              fontSize: '12px',
                              color: 'var(--color-text-tertiary)',
                            }}
                          >
                            {cmd.description}
                          </div>
                        )}
                      </div>
                      {globalIndex === selectedIndex && (
                        <ArrowRight size={16} style={{ color: 'var(--color-primary)' }} />
                      )}
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        )}

        {/* No Results */}
        {query && filteredCommands.length === 0 && (
          <div
            style={{
              padding: '32px',
              textAlign: 'center',
              color: 'var(--color-text-tertiary)',
              fontSize: '14px',
            }}
          >
            No commands found matching "{query}"
          </div>
        )}

        {/* Empty State */}
        {!query && (
          <div
            style={{
              padding: '32px',
              textAlign: 'center',
              color: '#94a3b8',
              fontSize: '13px',
            }}
          >
            Start typing to search commands...
            <div style={{ marginTop: '8px', fontSize: '11px' }}>
              Use ↑↓ to navigate, Enter to execute, Esc to close
            </div>
          </div>
        )}

        {/* Footer */}
        <div
          style={{
            padding: '8px 16px',
            borderTop: '1px solid var(--color-border)',
            fontSize: '11px',
            color: 'var(--color-text-tertiary)',
            display: 'flex',
            justifyContent: 'space-between',
          }}
        >
          <span>{filteredCommands.length} {filteredCommands.length === 1 ? 'command' : 'commands'}</span>
          <span>↑↓ Navigate • Enter Execute • Esc Close</span>
        </div>
      </div>
    </div>
  );
};
