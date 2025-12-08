import React, { useEffect, useRef, useState } from 'react';
import { Button } from '@sruja/ui';
import { validateNodeLabel } from '../utils/inputValidation';

interface InputModalProps {
    isOpen: boolean;
    title: string;
    initialValue?: string;
    placeholder?: string;
    onConfirm: (value: string) => void;
    onCancel: () => void;
}

export const InputModal: React.FC<InputModalProps> = ({
    isOpen,
    title,
    initialValue = '',
    placeholder = '',
    onConfirm,
    onCancel,
}) => {
    const [value, setValue] = useState(initialValue);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (isOpen) {
            setValue(initialValue);
            // Focus input after a short delay to allow render
            setTimeout(() => {
                inputRef.current?.focus();
                inputRef.current?.select();
            }, 50);
        }
    }, [isOpen, initialValue]);

    if (!isOpen) return null;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        // Validate and sanitize input
        const validation = validateNodeLabel(value);
        if (!validation.isValid) {
            // Show error (could use toast here)
            console.error('Validation error:', validation.error);
            return;
        }
        onConfirm(validation.sanitized || value);
    };

    return (
        <div style={{
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
        }}>
            <div style={{
                backgroundColor: 'var(--color-background)',
                borderRadius: '12px',
                boxShadow: 'var(--shadow-xl)',
                width: '100%',
                maxWidth: '400px',
                padding: '24px',
                animation: 'fadeIn 0.2s ease-out'
            }}>
                <h3 style={{
                    margin: '0 0 16px 0',
                    fontSize: '1.125rem',
                    fontWeight: 600,
                    color: 'var(--color-text-primary)'
                }}>{title}</h3>

                <form onSubmit={handleSubmit}>
                    <input
                        ref={inputRef}
                        type="text"
                        value={value}
                        onChange={(e) => setValue(e.target.value)}
                        placeholder={placeholder}
                        style={{
                            width: '100%',
                            padding: '10px 12px',
                            borderRadius: '6px',
                            border: '1px solid var(--color-border)',
                            fontSize: '0.95rem',
                            outline: 'none',
                            marginBottom: '20px',
                            color: 'var(--color-text-secondary)',
                            transition: 'border-color 0.2s'
                        }}
                        onFocus={(e) => e.target.style.borderColor = 'var(--color-primary)'}
                        onBlur={(e) => e.target.style.borderColor = 'var(--color-border)'}
                    />

                    <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px' }}>
                        <Button type="button" variant="ghost" onClick={onCancel}>
                            Cancel
                        </Button>
                        <Button type="submit" variant="primary" disabled={!value.trim()}>
                            Confirm
                        </Button>
                    </div>
                </form>
            </div>
        </div>
    );
};
