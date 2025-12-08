import { describe, it, expect, vi } from 'vitest';
import { createHandleApplyTemplate } from '../templateHandlers';

describe('templateHandlers', () => {
  it('applies template and calls updateDsl', async () => {
    const updateDsl = vi.fn().mockResolvedValue(undefined);
    const setToast = vi.fn();
    const apply = createHandleApplyTemplate({ updateDsl, setToast });
    await apply('web-app');
    expect(updateDsl).toHaveBeenCalled();
    expect(setToast).toHaveBeenCalledWith({ message: 'Applied web-app template', type: 'success' });
  });
});
