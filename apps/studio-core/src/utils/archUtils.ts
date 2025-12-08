import { ArchitectureJSON, SystemJSON, ContainerJSON, ComponentJSON, PersonJSON, DataStoreJSON, QueueJSON } from '@sruja/viewer';

type ArchNode = SystemJSON | ContainerJSON | ComponentJSON | PersonJSON | DataStoreJSON | QueueJSON;

export const findNode = (data: ArchitectureJSON, id: string): ArchNode | null => {
    const arch = data.architecture;

    // Check top-level
    if (arch.systems) {
        for (const sys of arch.systems) {
            if (sys.id === id) return sys;
            if (sys.containers) {
                for (const cont of sys.containers) {
                    if (`${sys.id}.${cont.id}` === id || cont.id === id) return cont;
                    if (cont.components) {
                        for (const comp of cont.components) {
                            if (`${sys.id}.${cont.id}.${comp.id}` === id || comp.id === id) return comp;
                        }
                    }
                }
            }
            // Check other system children
            if (sys.datastores) {
                for (const ds of sys.datastores) {
                    if (`${sys.id}.${ds.id}` === id || ds.id === id) return ds;
                }
            }
            if (sys.queues) {
                for (const q of sys.queues) {
                    if (`${sys.id}.${q.id}` === id || q.id === id) return q;
                }
            }
        }
    }

    if (arch.persons) {
        for (const p of arch.persons) {
            if (p.id === id) return p;
        }
    }

    // Check other top-level lists if needed (containers, datastores, queues)

    return null;
};

export const updateNode = (data: ArchitectureJSON, id: string, updateFn: (node: ArchNode) => ArchNode): ArchitectureJSON => {
    const newData = JSON.parse(JSON.stringify(data)); // Deep clone for simplicity
    const arch = newData.architecture;

    // Helper to process list
    const processList = (list: any[], prefix: string = '') => {
        if (!list) return;
        for (let i = 0; i < list.length; i++) {
            const item = list[i];
            const currentId = prefix ? `${prefix}.${item.id}` : item.id;

            if (currentId === id || item.id === id) {
                list[i] = updateFn(item);
                return true;
            }

            // Recurse
            if (item.containers) {
                if (processList(item.containers, currentId)) return true;
            }
            if (item.components) {
                if (processList(item.components, currentId)) return true;
            }
            if (item.datastores) {
                if (processList(item.datastores, currentId)) return true;
            }
            if (item.queues) {
                if (processList(item.queues, currentId)) return true;
            }
        }
        return false;
    };

    if (arch.systems) processList(arch.systems);
    if (arch.persons) processList(arch.persons);
    // Add other top-level lists if needed

    return newData;
};
