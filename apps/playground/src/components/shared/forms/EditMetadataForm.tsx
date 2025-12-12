// apps/playground/src/components/shared/forms/EditMetadataForm.tsx
import { useState, useEffect, useRef } from 'react';
import { useArchitectureStore } from '../../../stores';
import type { MetadataEntry } from '../../../types';
import { Button, Input } from '@sruja/ui';
import { SidePanel } from '../SidePanel';
import { X } from 'lucide-react';
import '../EditForms.css';

interface EditMetadataFormProps {
    isOpen: boolean;
    onClose: () => void;
    metadata?: MetadataEntry;
    metadataIndex?: number;
}

export function EditMetadataForm({ isOpen, onClose, metadata, metadataIndex }: EditMetadataFormProps) {
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);
    const formRef = useRef<HTMLFormElement>(null);
    const [key, setKey] = useState(metadata?.key || '');
    const [value, setValue] = useState(metadata?.value || '');
    const [isArray, setIsArray] = useState(!!metadata?.array);
    const [arrayValues, setArrayValues] = useState<string[]>(metadata?.array || ['']);
    const [errors, setErrors] = useState<Record<string, string>>({});

    useEffect(() => {
        if (isOpen) {
            setKey(metadata?.key || '');
            setValue(metadata?.value || '');
            setIsArray(!!metadata?.array);
            setArrayValues(metadata?.array || ['']);
            setErrors({});
        }
    }, [isOpen, metadata]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) {
                onClose();
            }
        };
        if (isOpen) {
            document.addEventListener('keydown', handleKeyDown);
            return () => document.removeEventListener('keydown', handleKeyDown);
        }
    }, [isOpen, onClose]);

    const addArrayItem = () => {
        setArrayValues([...arrayValues, '']);
    };

    const updateArrayItem = (index: number, val: string) => {
        const newArray = [...arrayValues];
        newArray[index] = val;
        setArrayValues(newArray);
    };

    const removeArrayItem = (index: number) => {
        setArrayValues(arrayValues.filter((_, i) => i !== index));
    };

    const validate = (): boolean => {
        const newErrors: Record<string, string> = {};
        if (!key.trim()) {
            newErrors.key = 'Key is required';
        }
        if (!isArray && !value.trim()) {
            newErrors.value = 'Value is required';
        }
        if (isArray && arrayValues.filter(v => v.trim()).length === 0) {
            newErrors.array = 'At least one array value is required';
        }
        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!validate()) return;

        try {
            await updateArchitecture((arch) => {
                if (!arch.architecture) return arch;
                const metadataList = [...(arch.architecture.archMetadata || [])];
                const newMetadata: MetadataEntry = {
                    key: key.trim(),
                    value: isArray ? undefined : (value.trim() || undefined),
                    array: isArray ? arrayValues.filter(v => v.trim()) : undefined,
                };

                if (metadata && metadataIndex !== undefined) {
                    metadataList[metadataIndex] = newMetadata;
                } else {
                    metadataList.push(newMetadata);
                }

                return {
                    ...arch,
                    architecture: {
                        ...arch.architecture,
                        archMetadata: metadataList,
                    },
                };
            });
            onClose();
        } catch (err) {
            console.error('Failed to update metadata:', err);
            setErrors({ submit: 'Failed to save metadata. Please try again.' });
        }
    };

    return (
        <SidePanel
            isOpen={isOpen}
            onClose={onClose}
            title={metadata ? 'Edit Metadata' : 'Add Metadata'}
            size="md"
            footer={
                <>
                    <Button variant="secondary" onClick={onClose} type="button">
                        Cancel
                    </Button>
                    <Button
                        variant="primary"
                        onClick={(e) => {
                            e.preventDefault();
                            handleSubmit(e as any);
                        }}
                    >
                        {metadata ? 'Update' : 'Add'}
                    </Button>
                </>
            }
        >
            <form ref={formRef} onSubmit={handleSubmit} className="edit-form">
                <Input
                    label="Key *"
                    value={key}
                    onChange={(e) => {
                        setKey(e.target.value);
                        if (errors.key) setErrors({ ...errors, key: '' });
                    }}
                    required
                    placeholder="e.g., team, owner, version"
                    error={errors.key}
                />
                <div className="form-group">
                    <label className="flex items-center gap-2 mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
                        <input
                            type="checkbox"
                            checked={isArray}
                            onChange={(e) => setIsArray(e.target.checked)}
                        />
                        <span>Array value (multiple items)</span>
                    </label>
                </div>
                {!isArray ? (
                    <Input
                        label="Value *"
                        value={value}
                        onChange={(e) => {
                            setValue(e.target.value);
                            if (errors.value) setErrors({ ...errors, value: '' });
                        }}
                        required
                        placeholder="Metadata value"
                        error={errors.value}
                    />
                ) : (
                    <div className="form-group">
                        <label className="block mb-1.5 text-sm font-medium text-[var(--color-text-secondary)]">
                            Values *
                        </label>
                        <div className="list-items">
                            {arrayValues.map((val, index) => (
                                <div key={index} className="list-item">
                                    <div>
                                        <Input
                                            value={val}
                                            onChange={(e) => updateArrayItem(index, e.target.value)}
                                            placeholder="Enter a value"
                                        />
                                    </div>
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => removeArrayItem(index)}
                                    >
                                        <X size={16} />
                                    </Button>
                                </div>
                            ))}
                            <Button type="button" variant="outline" onClick={addArrayItem}>
                                + Add Value
                            </Button>
                        </div>
                        {errors.array && (
                            <div className="mt-1 text-xs text-[var(--color-error-500)]">{errors.array}</div>
                        )}
                    </div>
                )}
            </form>
        </SidePanel>
    );
}
