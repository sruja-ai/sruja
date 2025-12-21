// apps/website/src/features/playground/components/LikeC4DiagramPreview.tsx
// LikeC4 diagram preview component - replaces DiagramPreview for LikeC4 format
// This component uses @likec4/diagram (v1.46.0) instead of @sruja/diagram

import { useMemo, useState, Component, type ReactNode } from 'react';
import type { SrujaModelDump } from '@sruja/shared';
import { LikeC4ModelProvider, LikeC4View } from '@likec4/diagram/bundle';
import { LikeC4Model } from '@likec4/core/model';

// Error boundary for LikeC4 view rendering errors
class LikeC4ViewErrorBoundary extends Component<
  { children: ReactNode; viewId: string; fallback?: ReactNode },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: ReactNode; viewId: string; fallback?: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidUpdate(prevProps: { viewId: string }) {
    // Reset error boundary when viewId changes
    if (prevProps.viewId !== this.props.viewId && this.state.hasError) {
      this.setState({ hasError: false, error: null });
    }
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }
      return (
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          color: 'var(--color-text-secondary)',
          fontSize: 14,
          padding: '2rem',
          textAlign: 'center'
        }}>
          <div>
            <p>Failed to render view "{this.props.viewId}"</p>
            {this.state.error?.message && (
              <p style={{ fontSize: 12, marginTop: '0.5rem', color: 'var(--color-text-tertiary)' }}>
                {this.state.error.message}
              </p>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

interface LikeC4DiagramPreviewProps {
  model: SrujaModelDump;
  viewId?: string;
}

/**
 * LikeC4-based diagram preview component
 * 
 * Uses @likec4/diagram bundle version which includes its own styles and handles
 * model setup internally from the provided JSON.
 */
export function LikeC4DiagramPreview({ model, viewId = 'index' }: LikeC4DiagramPreviewProps) {
  // Get available views from the model
  const availableViews = useMemo(() => {
    if (!model.views) return [];
    return Object.keys(model.views);
  }, [model]);

  // State for user-selected view
  const [selectedViewId, setSelectedViewId] = useState<string | null>(null);

  // Calculate safe view ID
  const activeViewId = useMemo(() => {
    // 1. Prefer user selection if valid
    if (selectedViewId && availableViews.includes(selectedViewId)) {
      return selectedViewId;
    }
    // 2. Fallback to prop viewId if valid
    if (viewId && availableViews.includes(viewId)) {
      return viewId;
    }
    // 3. Fallback to first available
    if (availableViews.length > 0) {
      return availableViews[0];
    }
    // 4. Last fallback
    return 'index';
  }, [selectedViewId, viewId, availableViews]);

  // Count elements and relations for stats
  const stats = useMemo(() => {
    const elementCount = Object.keys(model.elements || {}).length;
    const relationCount = (model.relations || []).length;
    return { elements: elementCount, relations: relationCount };
  }, [model]);

  // Convert JSON dump to LikeC4Model
  const likec4Model = useMemo(() => {
    const elements = model.elements || {};
    const elementIds = new Set(Object.keys(elements));

    // Filter relations to only include those with valid source and target references
    // LikeC4 expects relations to reference elements that exist in the elements map
    // Go backend now exports source/target as FqnRef objects: { model: "fqn" }
    const validRelations = (model.relations || []).filter(rel => {
      // Handle both FqnRef objects and legacy plain strings for backward compatibility
      const sourceFqn = typeof rel.source === 'object' ? rel.source.model : rel.source;
      const targetFqn = typeof rel.target === 'object' ? rel.target.model : rel.target;

      if (!sourceFqn || !targetFqn) {
        console.warn(`Skipping relation ${rel.id || 'unnamed'}: missing source or target`);
        return false;
      }

      // Check if both source and target exist in elements (exact match)
      const sourceExists = elementIds.has(sourceFqn);
      const targetExists = elementIds.has(targetFqn);

      if (!sourceExists || !targetExists) {
        // Try to find case-insensitive matches for better error messages
        const sourceMatch = Array.from(elementIds).find(id => id.toLowerCase() === sourceFqn.toLowerCase());
        const targetMatch = Array.from(elementIds).find(id => id.toLowerCase() === targetFqn.toLowerCase());

        console.warn(
          `Skipping relation ${rel.id || 'unnamed'}: ` +
          `source "${sourceFqn}" ${!sourceExists ? `(not found${sourceMatch ? `, did you mean "${sourceMatch}"?` : ''})` : 'found'} ` +
          `or target "${targetFqn}" ${!targetExists ? `(not found${targetMatch ? `, did you mean "${targetMatch}"?` : ''})` : 'found'}`
        );
        return false;
      }
      return true;
    });

    // Ensure model has required fields for LikeC4ModelDump
    const modelWithDeployments = {
      ...model,
      elements,
      relations: validRelations,
      deployments: { elements: {}, relations: {} }, // Required by LikeC4ModelDump
      project: {
        id: model._metadata?.name || 'sruja-project',
        name: model._metadata?.name || 'sruja-project'
      }, // Required by LikeC4ModelDump
    };

    try {
      return LikeC4Model.fromDump(modelWithDeployments as any);
    } catch (error) {
      console.error('Error creating LikeC4Model:', error);
      console.error('Model dump:', JSON.stringify(modelWithDeployments, null, 2));
      throw error;
    }
  }, [model]);


  return (
    <div style={{ width: '100%', height: '100%', minHeight: 400, position: 'relative' }}>
      {/* View selector */}
      {availableViews.length > 1 && (
        <div style={{ position: 'absolute', top: 8, right: 8, zIndex: 20 }}>
          <select
            value={activeViewId}
            onChange={(e) => setSelectedViewId(e.target.value)}
            style={{
              padding: '6px 10px',
              borderRadius: 6,
              border: '1px solid var(--color-border)',
              background: 'var(--color-background)',
              color: 'var(--color-text-primary)',
              fontSize: 12,
              cursor: 'pointer'
            }}
          >
            {availableViews.map(vid => (
              <option key={vid} value={vid}>
                {model.views?.[vid]?.title || vid}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* Stats display */}
      <div style={{
        position: 'absolute',
        top: 8,
        left: 8,
        zIndex: 20,
        padding: '6px 10px',
        borderRadius: 6,
        border: '1px solid var(--color-border)',
        background: 'var(--color-background)',
        color: 'var(--color-text-secondary)',
        fontSize: 12
      }}>
        {stats.elements} elements · {stats.relations} relations
      </div>

      {/* LikeC4 diagram */}
      <div style={{ width: '100%', height: '100%' }}>
        <LikeC4ViewErrorBoundary
          viewId={activeViewId}
          fallback={
            <div style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              color: 'var(--color-text-secondary)',
              fontSize: 14,
              padding: '2rem',
              textAlign: 'center'
            }}>
              <div>
                <p>Unable to render view "{activeViewId}"</p>
                <p style={{ fontSize: 12, marginTop: '0.5rem', color: 'var(--color-text-tertiary)' }}>
                  The view may not be layouted or the model may not contain layout information.
                </p>
                {availableViews.length > 0 && (
                  <p style={{ fontSize: 12, marginTop: '0.5rem', color: 'var(--color-text-tertiary)' }}>
                    Available views: {availableViews.join(', ')}
                  </p>
                )}
              </div>
            </div>
          }
        >
          <LikeC4ModelProvider likec4model={likec4Model}>
            <LikeC4View viewId={activeViewId} />
          </LikeC4ModelProvider>
        </LikeC4ViewErrorBoundary>
      </div>
    </div>
  );
}
