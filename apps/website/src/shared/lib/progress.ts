// apps/website/src/shared/lib/progress.ts
import { createStorage } from '../utils/storage';
import { STORAGE_KEYS } from '../constants/storage';

const completedStorage = createStorage<string[]>(STORAGE_KEYS.COMPLETED_CHALLENGES, []);

export function getCompleted(): string[] {
  return completedStorage.get();
}

export function isCompleted(slug: string): boolean {
  return getCompleted().includes(slug);
}

export function markCompleted(slug: string): void {
  const completed = getCompleted();
  const set = new Set(completed);
  set.add(slug);
  completedStorage.set(Array.from(set));
}
