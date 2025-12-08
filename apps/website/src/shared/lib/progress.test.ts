import { describe, it, expect, beforeEach } from 'vitest';
import { getCompleted, isCompleted, markCompleted } from './progress';
import { STORAGE_KEYS } from '../constants/storage';

describe('progress', () => {
  beforeEach(() => {
    localStorage.removeItem(STORAGE_KEYS.COMPLETED_CHALLENGES);
  });

  it('initially returns empty completed list', () => {
    expect(getCompleted()).toEqual([]);
  });

  it('marks a challenge as completed and checks status', () => {
    expect(isCompleted('intro')).toBe(false);
    markCompleted('intro');
    expect(isCompleted('intro')).toBe(true);
    // idempotent
    markCompleted('intro');
    expect(getCompleted()).toEqual(['intro']);
  });
});
