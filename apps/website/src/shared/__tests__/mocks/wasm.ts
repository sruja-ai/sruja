// apps/website/src/shared/__tests__/mocks/wasm.ts
import { vi } from 'vitest';

export interface MockWasmApi {
  parseDslToJson: ReturnType<typeof vi.fn>;
  // Add other WASM methods as needed
}

export function createMockWasm(): MockWasmApi {
  return {
    parseDslToJson: vi.fn().mockResolvedValue(JSON.stringify({
      architecture: {
        name: 'Test Architecture',
        systems: [],
        persons: [],
      },
    })),
  };
}
