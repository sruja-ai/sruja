/**
 * useTransition Hook
 * 
 * Provides smooth animated transitions between layout states in React Flow.
 * Uses the @sruja/layout transitions engine.
 */

import { useCallback, useRef, useState } from 'react';
import { useReactFlow } from '@xyflow/react';
import type { Node, Edge } from '@xyflow/react';
import {
    createTransition,
    type TransitionState,
    type TransitionNode,
    type TransitionEdge,
    type AnimationFrame
} from '@sruja/layout';
import type { C4NodeData } from '../types';

export interface UseTransitionOptions {
    /** Animation duration in milliseconds */
    duration?: number;
    /** Callback when transition completes */
    onComplete?: () => void;
}

export interface UseTransitionResult {
    /** Start a transition from current state to new state */
    startTransition: (
        newNodes: Node<C4NodeData>[],
        newEdges: Edge[]
    ) => void;
    /** Whether a transition is currently in progress */
    isAnimating: boolean;
    /** Cancel the current animation */
    cancel: () => void;
}

/**
 * Convert React Flow nodes to transition nodes
 */
function nodesToTransitionState(
    nodes: Node<C4NodeData>[],
    edges: Edge[],
    viewport: { x: number; y: number; zoom: number }
): TransitionState {
    const transitionNodes = new Map<string, TransitionNode>();

    for (const node of nodes) {
        transitionNodes.set(node.id, {
            id: node.id,
            position: node.position,
            size: {
                width: typeof node.width === 'number' ? node.width : 200,
                height: typeof node.height === 'number' ? node.height : 120
            },
            opacity: 1,
            scale: 1
        });
    }

    const transitionEdges: TransitionEdge[] = edges.map(edge => ({
        id: edge.id,
        points: (edge.data?.points as { x: number; y: number }[]) ?? [
            { x: 0, y: 0 },
            { x: 100, y: 100 }
        ],
        opacity: 1
    }));

    return {
        nodes: transitionNodes,
        edges: transitionEdges,
        viewport
    };
}

/**
 * Apply animation frame to React Flow
 */
function applyFrame(
    frame: AnimationFrame,
    originalNodes: Node<C4NodeData>[],
    originalEdges: Edge[],
    setNodes: (updater: (nodes: Node<C4NodeData>[]) => Node<C4NodeData>[]) => void,
    setEdges: (updater: (edges: Edge[]) => Edge[]) => void,
    setViewport: (viewport: { x: number; y: number; zoom: number }) => void
) {
    setNodes(() =>
        originalNodes.map(node => {
            const animatedNode = frame.nodes.get(node.id);
            if (animatedNode) {
                return {
                    ...node,
                    position: animatedNode.position,
                    style: {
                        ...node.style,
                        opacity: animatedNode.opacity,
                        transform: `scale(${animatedNode.scale})`
                    }
                };
            }
            return node;
        })
    );

    setEdges(() =>
        originalEdges.map(edge => {
            const animatedEdge = frame.edges.find(e => e.id === edge.id);
            if (animatedEdge) {
                return {
                    ...edge,
                    data: {
                        ...edge.data,
                        points: animatedEdge.points
                    },
                    style: {
                        ...edge.style,
                        opacity: animatedEdge.opacity,
                        strokeDashoffset: animatedEdge.strokeDashoffset
                    }
                };
            }
            return edge;
        })
    );

    setViewport(frame.viewport);
}

/**
 * Hook for smooth animated transitions between layout states
 */
export function useTransition(options: UseTransitionOptions = {}): UseTransitionResult {
    const { duration = 300, onComplete } = options;
    const { getNodes, getEdges, setNodes, setEdges, getViewport, setViewport } = useReactFlow<Node<C4NodeData>, Edge>();

    const [isAnimating, setIsAnimating] = useState(false);
    const animationRef = useRef<number | null>(null);
    const startTimeRef = useRef<number>(0);
    const transitionFnRef = useRef<((t: number) => AnimationFrame) | null>(null);
    const targetNodesRef = useRef<Node<C4NodeData>[]>([]);
    const targetEdgesRef = useRef<Edge[]>([]);

    const cancel = useCallback(() => {
        if (animationRef.current !== null) {
            cancelAnimationFrame(animationRef.current);
            animationRef.current = null;
        }
        setIsAnimating(false);
    }, []);

    const startTransition = useCallback(
        (newNodes: Node<C4NodeData>[], newEdges: Edge[]) => {
            // Cancel any existing animation
            cancel();

            const currentNodes = getNodes();
            const currentEdges = getEdges();
            const currentViewport = getViewport();

            // Create transition states
            const oldState = nodesToTransitionState(currentNodes, currentEdges, currentViewport);

            // Calculate new viewport to fit new nodes
            const newViewport = currentViewport; // Keep same viewport for now

            const newState = nodesToTransitionState(newNodes, newEdges, newViewport);

            // Create lazy transition function
            const transitionFn = createTransition(oldState, newState);
            transitionFnRef.current = transitionFn;
            targetNodesRef.current = newNodes;
            targetEdgesRef.current = newEdges;

            setIsAnimating(true);
            startTimeRef.current = Date.now();

            const animate = () => {
                const elapsed = Date.now() - startTimeRef.current;
                const t = Math.min(elapsed / duration, 1);

                if (transitionFnRef.current) {
                    const frame = transitionFnRef.current(t);
                    applyFrame(
                        frame,
                        targetNodesRef.current,
                        targetEdgesRef.current,
                        setNodes,
                        setEdges,
                        setViewport
                    );
                }

                if (t < 1) {
                    animationRef.current = requestAnimationFrame(animate);
                } else {
                    // Animation complete - apply final state directly
                    setNodes(() => targetNodesRef.current);
                    setEdges(() => targetEdgesRef.current);
                    setIsAnimating(false);
                    animationRef.current = null;
                    onComplete?.();
                }
            };

            animationRef.current = requestAnimationFrame(animate);
        },
        [cancel, getNodes, getEdges, getViewport, setNodes, setEdges, setViewport, duration, onComplete]
    );

    return {
        startTransition,
        isAnimating,
        cancel
    };
}
