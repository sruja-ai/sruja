
import { useEffect, useRef } from 'react';
import { persistence } from '../utils/persistence';

export function useAutosave(dsl: string, delay: number = 1000) {
    const timeoutRef = useRef<NodeJS.Timeout | null>(null);
    const lastSavedRef = useRef<string>(dsl);

    useEffect(() => {
        if (dsl === lastSavedRef.current) return;

        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }

        timeoutRef.current = setTimeout(() => {
            persistence.saveLocal(dsl);
            lastSavedRef.current = dsl;
            console.log('Autosaved DSL to local storage');
        }, delay);

        return () => {
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }
        };
    }, [dsl, delay]);
}
