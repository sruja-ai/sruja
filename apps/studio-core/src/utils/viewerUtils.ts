// apps/studio-core/src/utils/viewerUtils.ts
import type { ViewerInstance } from '@sruja/viewer';
import type { ArchitectureJSON } from '@sruja/viewer';
import type { WasmApi } from '@sruja/shared';
import { validatePropertiesUpdate } from './inputValidation';

export async function updateViewer(
  currentDsl: string,
  api: WasmApi,
  viewer: ViewerInstance,
  setArchData: (data: ArchitectureJSON | null) => void,
  setValidationStatus: (status: {
    isValid: boolean;
    errors: number;
    warnings: number;
    lastError?: string;
    diagnostics?: any[];
  }) => void,
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void,
  isUpdatingRef?: React.MutableRefObject<boolean>,
  containerElement?: HTMLElement | null
) {
  if (isUpdatingRef) {
    isUpdatingRef.current = true;
  }
  try {
    const parsedJson = await api.parseDslToJson(currentDsl);
    const parsed = JSON.parse(parsedJson);

    // Merge existing layout metadata if available
    if (viewer.getLayout) {
      const currentLayout = viewer.getLayout();
      if (Object.keys(currentLayout).length > 0) {
        if (!parsed.metadata) parsed.metadata = {};
        (parsed.metadata as any).layout = currentLayout;
      }
    }

    setArchData(parsed);
    
    // Load the data into the viewer
    console.log('Loading architecture data into viewer:', parsed);
    await viewer.load(parsed);
    console.log('Viewer load completed, cy exists:', !!viewer.cy, 'elements:', viewer.cy?.elements().length);
    
    // Wait for Cytoscape to be fully ready and ensure container is visible
    if (viewer.cy && !viewer.cy.destroyed()) {
      console.log('Cytoscape instance exists, ensuring proper rendering');
      
      // Get the container element to check dimensions
      // Use cy.container() to get the actual DOM element
      const containerEl = containerElement || (viewer.cy?.container() as HTMLElement | undefined);
      
      // Wait for container to have proper dimensions
      const waitForDimensions = async () => {
        if (!containerEl) return;
        
        for (let i = 0; i < 20; i++) {
          const rect = containerEl.getBoundingClientRect();
          console.log(`Waiting for container dimensions (attempt ${i + 1}):`, rect.width, 'x', rect.height);
          
          if (rect.width > 0 && rect.height > 0) {
            console.log('Container has proper dimensions:', rect.width, 'x', rect.height);
            return true;
          }
          
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        
        console.warn('Container still has zero dimensions after waiting');
        return false;
      };
      
      const hasDimensions = await waitForDimensions();
      
      // Wait for ready state and ensure proper rendering
      await new Promise<void>((resolve) => {
        const cy = viewer.cy!;
        let resolved = false;
        let retryCount = 0;
        const maxRetries = 10; // Maximum retries for zero dimensions
        
        const doResize = () => {
          if (resolved || !cy || cy.destroyed()) return;
          
          // Check container dimensions again
          if (containerEl) {
            const rect = containerEl.getBoundingClientRect();
            console.log('Resizing viewer, container dimensions:', rect.width, 'x', rect.height);
            
            if (rect.width === 0 || rect.height === 0) {
              retryCount++;
              if (retryCount >= maxRetries) {
                logger.warn('Container still has zero dimensions after max retries', { component: 'studio', action: 'resize_viewer', retries: maxRetries });
                resolved = true;
                resolve();
                return;
              }
              logger.warn('Container still has zero dimensions, will retry', { component: 'studio', action: 'resize_viewer', retryCount, maxRetries });
              setTimeout(doResize, 200);
              return;
            }
          }
          
          try {
            console.log('Calling cy.resize() and cy.fit()');
            cy.resize();
            cy.fit(undefined, 80);
            
            // Verify elements are visible
            const elements = cy.elements();
            console.log('Total elements:', elements.length);
            elements.forEach((el, idx) => {
              if (idx < 3) { // Log first 3 elements
                const pos = el.position();
                console.log(`Element ${el.id()}: position (${pos.x}, ${pos.y}), visible: ${el.visible()}`);
              }
            });
            
            resolved = true;
            resolve();
          } catch (e) {
            logger.warn('Resize failed', {
              component: 'studio',
              action: 'resize_viewer',
              errorType: e instanceof Error ? e.constructor.name : 'unknown',
              error: e instanceof Error ? e.message : String(e),
            });
            resolved = true;
            resolve();
          }
        };
        
        // Only proceed if we have dimensions or if waitForDimensions gave up
        if (!hasDimensions && containerEl) {
          const rect = containerEl.getBoundingClientRect();
          if (rect.width === 0 || rect.height === 0) {
            logger.warn('Container has zero dimensions, will try to resize anyway', { component: 'studio', action: 'resize_viewer' });
          }
        }
        
        // Try ready callback (only once)
        let readyCalled = false;
        cy.ready(() => {
          if (readyCalled) return;
          readyCalled = true;
          console.log('Cytoscape ready callback');
          doResize();
        });
        
        // Listen for layoutstop (but only once per layout)
        let layoutStopHandler: (() => void) | null = null;
        layoutStopHandler = () => {
          if (resolved) {
            // Remove listener after first successful resize
            cy.off('layoutstop', layoutStopHandler!);
            return;
          }
          console.log('Layout stopped');
          doResize();
        };
        cy.on('layoutstop', layoutStopHandler);
        
        // Immediate try after a short delay
        setTimeout(doResize, 100);
        
        // Timeout fallback - force resolve after 5 seconds max
        setTimeout(() => {
          if (!resolved) {
            console.log('Timeout fallback: forcing resolve (max wait time exceeded)');
            resolved = true;
            resolve();
          }
        }, 5000);
      });
    } else {
      logger.warn('Viewer cy not available after load', { component: 'studio', action: 'load_viewer' });
    }

    // Validation passed (parsing succeeded)
    // Now fetch diagnostics (semantic checks, suggestions)
    let diagnostics: any[] = [];
    try {
      if (api.getDiagnostics) {
        const diagJson = await api.getDiagnostics(currentDsl);
        const diagResult = JSON.parse(diagJson);
        if (diagResult.ok && diagResult.data) {
          diagnostics = JSON.parse(diagResult.data);
        }
      }
    } catch (e) {
      logger.warn('Failed to fetch diagnostics', {
      component: 'studio',
      action: 'fetch_diagnostics',
      errorType: e instanceof Error ? e.constructor.name : 'unknown',
      error: e instanceof Error ? e.message : String(e),
    });
    }

    const errors = diagnostics.filter((d: any) => d.severity === 'Error').length;
    const warnings = diagnostics.filter((d: any) => d.severity === 'Warning').length;

    setValidationStatus({
      isValid: errors === 0,
      errors,
      warnings,
      lastError: undefined,
      diagnostics
    });
  } catch (e) {
    logger.error('Parse error', {
      component: 'studio',
      action: 'parse_dsl',
      errorType: e instanceof Error ? e.constructor.name : 'unknown',
      error: e instanceof Error ? e.message : String(e),
    });
    const errorMessage = e instanceof Error ? e.message : String(e);

    // Parse error message to extract useful information
    let errorCount = 1;
    let warningCount = 0;
    let displayMessage = errorMessage;

    // Try to extract line numbers and make message more user-friendly
    const lineMatch = errorMessage.match(/line\s+(\d+)/i) || errorMessage.match(/at\s+line\s+(\d+)/i);
    if (lineMatch) {
      const lineNum = lineMatch[1];
      displayMessage = `Parse error at line ${lineNum}: ${errorMessage.replace(/.*line\s+\d+[:\s]*/i, '').replace(/.*at\s+line\s+\d+[:\s]*/i, '')}`;
    }

    // Count multiple errors if message contains multiple error indicators
    const errorMatches = errorMessage.match(/error/gi);
    if (errorMatches) {
      errorCount = errorMatches.length;
    }

    // Check for warnings
    const warningMatches = errorMessage.match(/warning/gi);
    if (warningMatches) {
      warningCount = warningMatches.length;
    }

    // Truncate very long error messages
    if (displayMessage.length > 150) {
      displayMessage = displayMessage.substring(0, 147) + '...';
    }

    setToast({ message: displayMessage, type: 'error' });
    setValidationStatus({ isValid: false, errors: errorCount, warnings: warningCount, lastError: displayMessage, diagnostics: [] });
  }
}

export async function syncDiagramToDsl(
  viewerRef: React.RefObject<ViewerInstance | null>,
  wasmApiRef: React.RefObject<WasmApi | null>,
  setDsl: (dsl: string) => void,
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void,
  setArchData?: (data: ArchitectureJSON | null) => void
) {
  if (!viewerRef.current || !wasmApiRef.current) return;

  // Get JSON with layout metadata from viewer
  const json = viewerRef.current.toJSON();
  const jsonStr = JSON.stringify(json);

  try {
    const newDsl = await wasmApiRef.current.printJsonToDsl(jsonStr);
    setDsl(newDsl);
    // Also update archData directly to ensure UI refreshes immediately
    if (setArchData) {
      setArchData(json);
    }
  } catch (e) {
    logger.error('Failed to sync diagram to DSL', {
      component: 'studio',
      action: 'sync_diagram_to_dsl',
      errorType: e instanceof Error ? e.constructor.name : 'unknown',
      error: e instanceof Error ? e.message : String(e),
    });
    setToast({ message: 'Failed to sync diagram to DSL', type: 'error' });
  }
}

export async function handlePropertiesUpdate(
  newData: ArchitectureJSON,
  wasmApiRef: React.RefObject<WasmApi | null>,
  viewerRef: React.RefObject<ViewerInstance | null>,
  selectedNodeId: string | null,
  setArchData: (data: ArchitectureJSON | null) => void,
  setDsl: (dsl: string) => void,
  setToast: (toast: { message: string; type: 'success' | 'error' | 'info' } | null) => void,
  isUpdatingPropertiesRef?: React.MutableRefObject<boolean>
) {
  const wasmApi = wasmApiRef.current;
  const viewer = viewerRef.current;
  if (!wasmApi || !viewer) return;

  // Validate properties update data
  // Note: newData is ArchitectureJSON, but we validate string fields within it
  // The actual validation happens when we convert to DSL, but we can add basic checks here
  if (!newData || typeof newData !== 'object') {
    setToast({ message: 'Invalid properties data', type: 'error' });
    return;
  }

  // Set flag FIRST to prevent useEffect from triggering
  if (isUpdatingPropertiesRef) {
    isUpdatingPropertiesRef.current = true;
  }

  // Preserve zoom and pan before update
  const cy = viewer.cy;
  let savedZoom = 1;
  let savedPan = { x: 0, y: 0 };
  if (cy && !cy.destroyed()) {
    savedZoom = cy.zoom();
    savedPan = cy.pan();
  }

  // Update archData state (this will be debounced by the PropertiesPanel)
  // But we need to update it for DSL sync
  // Flag is already set, so useEffect won't trigger
  setArchData(newData);

  // Sync to DSL
  try {
    const newDsl = await wasmApi.printJsonToDsl(JSON.stringify(newData));
    setDsl(newDsl);
    
    if (viewer && selectedNodeId && cy && !cy.destroyed()) {
      // Always try to update the node directly in Cytoscape without reloading
      const node = cy.getElementById(selectedNodeId);
      if (node && node.length > 0) {
        // Find the updated node data
        const findNodeInData = (data: ArchitectureJSON, id: string): any => {
          const arch = data.architecture || {};
          // Check persons
          if (arch.persons) {
            const found = arch.persons.find((p: any) => p.id === id);
            if (found) return found;
          }
          // Check systems
          if (arch.systems) {
            for (const system of arch.systems) {
              if (system.id === id) return system;
              // Check containers
              if (system.containers) {
                for (const container of system.containers) {
                  if (container.id === id) return container;
                  // Check components
                  if (container.components) {
                    const found = container.components.find((c: any) => c.id === id);
                    if (found) return found;
                  }
                }
              }
            }
          }
          return null;
        };

        const updatedNodeData = findNodeInData(newData, selectedNodeId);
        if (updatedNodeData) {
          // Update node data directly in Cytoscape
          node.data('label', updatedNodeData.label || updatedNodeData.id);
          if (updatedNodeData.description !== undefined) {
            node.data('description', updatedNodeData.description || '');
          }
          if (updatedNodeData.technology !== undefined) {
            node.data('technology', updatedNodeData.technology || '');
          }
          
          // Restore selection WITHOUT calling selectNode (which triggers fit/zoom)
          cy.nodes().unselect();
          node.select();
          
          // Restore zoom and pan AFTER selection (preserve user's view)
          // Use immediate zoom/pan (not animated) to prevent any zoom effects
          cy.stop(); // Stop any ongoing animations
          cy.zoom(savedZoom);
          cy.pan(savedPan);
          
          // Reset flag after successful direct update
          if (isUpdatingPropertiesRef) {
            setTimeout(() => {
              isUpdatingPropertiesRef.current = false;
            }, 100);
          }
          
          // Don't reload - we've updated the node directly
          return;
        }
      }
    }
    
    // Fallback: Only reload if direct update didn't work (shouldn't happen normally)
    // But preserve zoom/pan even in fallback
    await viewer.load(newData);
    
    // Restore zoom and pan after reload WITHOUT calling selectNode (which triggers fit)
    const newCy = viewer.cy;
    if (newCy && !newCy.destroyed()) {
      setTimeout(() => {
        if (selectedNodeId) {
          const node = newCy.getElementById(selectedNodeId);
          if (node && node.length > 0) {
            newCy.nodes().unselect();
            node.select();
          }
        }
        // Stop any animations and restore zoom/pan immediately
        newCy.stop(); // Stop any ongoing animations (including fit animations)
        newCy.zoom(savedZoom);
        newCy.pan(savedPan);
        
        // Reset flag after reload
        if (isUpdatingPropertiesRef) {
          setTimeout(() => {
            isUpdatingPropertiesRef.current = false;
          }, 100);
        }
      }, 100);
    } else if (isUpdatingPropertiesRef) {
      // Reset flag even if cy is not available
      setTimeout(() => {
        isUpdatingPropertiesRef.current = false;
      }, 100);
    }
  } catch (error) {
    logger.error('Failed to sync properties update to DSL', {
      component: 'studio',
      action: 'sync_properties_to_dsl',
      errorType: error instanceof Error ? error.constructor.name : 'unknown',
      error: error instanceof Error ? error.message : String(error),
    });
    setToast({ message: 'Failed to update properties', type: 'error' });
  }
}



