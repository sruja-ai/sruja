// apps/playground/src/components/shared/forms/EditRequirementForm.tsx
import { useState, useEffect } from 'react';
import { useArchitectureStore } from '../../../stores';
import type { RequirementJSON } from '../../../types';
import { Button, Input, Listbox, Textarea } from '@sruja/ui';
import type { ListOption } from '@sruja/ui';
import { SidePanel } from '../SidePanel';

import { REQUIREMENT_TYPES } from './constants';
import '../EditForms.css';

interface EditRequirementFormProps {
    isOpen: boolean;
    onClose: () => void;
    requirement?: RequirementJSON;
}

export function EditRequirementForm({ isOpen, onClose, requirement }: EditRequirementFormProps) {
    const updateArchitecture = useArchitectureStore((s) => s.updateArchitecture);

    const [id, setId] = useState(requirement?.id || '');
    const [type, setType] = useState<ListOption | null>(
        REQUIREMENT_TYPES.find((t) => t.id === (requirement?.type || 'functional')) || REQUIREMENT_TYPES[0]
    );
    const [title, setTitle] = useState(requirement?.title || '');
    const [description, setDescription] = useState(requirement?.description || '');
    const [errors, setErrors] = useState<Record<string, string>>({});

    useEffect(() => {
        if (isOpen) {
            setId(requirement?.id || '');
            setType(
                REQUIREMENT_TYPES.find((t) => t.id === (requirement?.type || 'functional')) || REQUIREMENT_TYPES[0]
            );
            setTitle(requirement?.title || '');
            setDescription(requirement?.description || '');
            setErrors({});
        }
    }, [isOpen, requirement]);

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

    const validate = (): boolean => {
        const newErrors: Record<string, string> = {};
        if (!id.trim()) {
            newErrors.id = 'ID is required';
        } else if (!/^[A-Za-z0-9_-]+$/.test(id.trim())) {
            newErrors.id = 'ID can only contain letters, numbers, hyphens, and underscores';
        }
        if (!title.trim() && !description.trim()) {
            newErrors.title = 'Title or description is required';
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
                const requirements = [...(arch.architecture.requirements || [])];
                const newRequirement: RequirementJSON = {
                    id: id.trim(),
                    type: type?.id as RequirementJSON['type'],
                    title: title.trim() || undefined,
                    description: description.trim() || undefined,
                };

                if (requirement) {
                    const index = requirements.findIndex((r) => r.id === requirement.id);
                    if (index >= 0) {
                        requirements[index] = newRequirement;
                    }
                } else {
                    requirements.push(newRequirement);
                }

                return {
                    ...arch,
                    architecture: {
                        ...arch.architecture,
                        requirements,
                    },
                };
            });
            onClose();
        } catch (err) {
            console.error('Failed to update requirement:', err);
            setErrors({ submit: 'Failed to save requirement. Please try again.' });
        }
    };

    return (
        <SidePanel
            isOpen={isOpen}
            onClose={onClose}
            title={requirement ? 'Edit Requirement' : 'Add Requirement'}
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
                        {requirement ? 'Update' : 'Add'}
                    </Button>
                </>
            }
        >
            <form onSubmit={handleSubmit} className="edit-form">
                <Input
                    name="id"
                    label="ID *"
                    value={id}
                    onChange={(e) => {
                        setId(e.target.value);
                        if (errors.id) setErrors({ ...errors, id: '' });
                    }}
                    required
                    placeholder="R1"
                    error={errors.id}
                />
                <Listbox
                    label="Type"
                    options={REQUIREMENT_TYPES}
                    value={type}
                    onChange={setType}
                />
                <Input
                    name="title"
                    label="Title"
                    value={title}
                    onChange={(e) => {
                        setTitle(e.target.value);
                        if (errors.title) setErrors({ ...errors, title: '' });
                    }}
                    placeholder="Requirement title"
                    error={errors.title}
                />
                <Textarea
                    name="description"
                    label="Description"
                    value={description}
                    onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setDescription(e.target.value)}
                    rows={4}
                    placeholder="Requirement description"
                />
                {errors.submit && (
                    <div className="text-sm text-[var(--color-error-500)] mt-2">{errors.submit}</div>
                )}
            </form>
        </SidePanel>
    );
}
