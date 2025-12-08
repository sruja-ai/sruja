
const STORAGE_KEY = 'sruja_studio_dsl_autosave';

export const persistence = {
    saveLocal: async (dsl: string): Promise<void> => {
        try {
            localStorage.setItem(STORAGE_KEY, dsl);
        } catch (error) {
            console.error('Failed to save DSL locally:', error);
        }
    },

    loadLocal: async (): Promise<string | null> => {
        try {
            return localStorage.getItem(STORAGE_KEY);
        } catch (error) {
            console.error('Failed to load DSL locally:', error);
            return null;
        }
    },

    clearLocal: async (): Promise<void> => {
        try {
            localStorage.removeItem(STORAGE_KEY);
        } catch (error) {
            console.error('Failed to clear local DSL:', error);
        }
    }
};
