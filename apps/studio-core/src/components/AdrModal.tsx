import React, { useState, useEffect } from 'react';
import { Button } from '@sruja/ui';

interface AdrModalProps {
    isOpen: boolean;
    onConfirm: (data: AdrData) => void;
    onCancel: () => void;
}

export interface AdrData {
    title: string;
    status: string;
    context: string;
    decision: string;
    consequences: string;
}

export const AdrModal: React.FC<AdrModalProps> = ({ isOpen, onConfirm, onCancel }) => {
    const [title, setTitle] = useState('');
    const [status, setStatus] = useState('proposed');
    const [context, setContext] = useState('');
    const [decision, setDecision] = useState('');
    const [consequences, setConsequences] = useState('');

    useEffect(() => {
        if (isOpen) {
            setTitle('');
            setStatus('proposed');
            setContext('');
            setDecision('');
            setConsequences('');
        }
    }, [isOpen]);

    if (!isOpen) return null;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!title.trim()) return;
        onConfirm({
            title,
            status,
            context,
            decision,
            consequences
        });
    };

    const overlayStyle: React.CSSProperties = {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'var(--overlay-scrim)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
        backdropFilter: 'blur(2px)',
    };

    const modalStyle: React.CSSProperties = {
        backgroundColor: 'var(--color-background)',
        borderRadius: '12px',
        padding: '24px',
        width: '100%',
        maxWidth: '600px',
        boxShadow: 'var(--shadow-xl)',
        animation: 'modalFadeIn 0.2s ease-out',
        maxHeight: '90vh',
        overflowY: 'auto',
    };

    const headerStyle: React.CSSProperties = {
        marginBottom: '20px',
    };

    const titleStyle: React.CSSProperties = {
        margin: 0,
        fontSize: '1.25rem',
        fontWeight: 600,
        color: 'var(--color-text-primary)',
    };

    const formGroupStyle: React.CSSProperties = {
        marginBottom: '16px',
    };

    const labelStyle: React.CSSProperties = {
        display: 'block',
        fontSize: '0.875rem',
        fontWeight: 500,
        color: 'var(--color-text-secondary)',
        marginBottom: '6px',
    };

    const inputStyle: React.CSSProperties = {
        width: '100%',
        padding: '10px 12px',
        borderRadius: '6px',
        border: '1px solid var(--color-border)',
        fontSize: '0.95rem',
        outline: 'none',
        transition: 'border-color 0.2s',
        boxSizing: 'border-box',
    };

    const textareaStyle: React.CSSProperties = {
        ...inputStyle,
        minHeight: '80px',
        resize: 'vertical',
        fontFamily: 'inherit',
    };







    return (
        <div style={overlayStyle} onClick={onCancel}>
            <div style={modalStyle} onClick={e => e.stopPropagation()}>
                <div style={headerStyle}>
                    <h2 style={titleStyle}>New Architecture Decision Record</h2>
                </div>
                <form onSubmit={handleSubmit}>
                    <div style={formGroupStyle}>
                        <label style={labelStyle}>Title *</label>
                        <input
                            autoFocus
                            style={inputStyle}
                            value={title}
                            onChange={e => setTitle(e.target.value)}
                            placeholder="e.g., Use PostgreSQL for primary storage"
                        />
                    </div>

                    <div style={formGroupStyle}>
                        <label style={labelStyle}>Status</label>
                        <select
                            style={inputStyle}
                            value={status}
                            onChange={e => setStatus(e.target.value)}
                        >
                            <option value="proposed">Proposed</option>
                            <option value="accepted">Accepted</option>
                            <option value="rejected">Rejected</option>
                            <option value="deprecated">Deprecated</option>
                            <option value="superseded">Superseded</option>
                        </select>
                    </div>

                    <div style={formGroupStyle}>
                        <label style={labelStyle}>Context</label>
                        <textarea
                            style={textareaStyle}
                            value={context}
                            onChange={e => setContext(e.target.value)}
                            placeholder="What is the issue that we're seeing that is motivating this decision or change?"
                        />
                    </div>

                    <div style={formGroupStyle}>
                        <label style={labelStyle}>Decision</label>
                        <textarea
                            style={textareaStyle}
                            value={decision}
                            onChange={e => setDecision(e.target.value)}
                            placeholder="What is the change that we're proposing and/or doing?"
                        />
                    </div>

                    <div style={formGroupStyle}>
                        <label style={labelStyle}>Consequences</label>
                        <textarea
                            style={textareaStyle}
                            value={consequences}
                            onChange={e => setConsequences(e.target.value)}
                            placeholder="What becomes easier or more difficult to do and any risks introduced?"
                        />
                    </div>

                    <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12, marginTop: 24 }}>
                        <Button type="button" variant="ghost" onClick={onCancel}>
                            Cancel
                        </Button>
                        <Button type="submit" variant="primary" disabled={!title.trim()}>
                            Create ADR
                        </Button>
                    </div>
                </form>
            </div>
        </div>
    );
};
