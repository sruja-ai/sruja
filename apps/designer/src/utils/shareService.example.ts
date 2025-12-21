// apps/playground/src/utils/shareService.example.ts
// Example usage of shareService with different storage adapters

import {
  shareService,
  LocalStorageAdapter,
  IndexedDBAdapter,
  BackendAPIAdapter,
  // CompositeStorageAdapter, // Unused
} from "./shareService";

// Example 1: Use localStorage only (default)
// shareService is already configured with LocalStorageAdapter

// Example 2: Use IndexedDB only (for larger storage)
// shareService.setStorageAdapter(new IndexedDBAdapter());

// Example 3: Use backend API only
// shareService.setStorageAdapter(new BackendAPIAdapter("https://api.example.com/shares"));

// Example 4: Use composite storage (try localStorage, then IndexedDB, then backend)
// This provides fallback and sync capabilities
shareService.setStorageAdapter(
  new LocalStorageAdapter(),
  new IndexedDBAdapter(),
  new BackendAPIAdapter("/api/shares")
);

// Example 5: Custom storage adapter
// class CustomStorageAdapter implements ShareStorageAdapter {
//   async get(shareId: string): Promise<ShareEntry | null> {
//     // Your custom logic
//   }
//   async set(shareId: string, entry: ShareEntry): Promise<void> {
//     // Your custom logic
//   }
//   async delete(shareId: string): Promise<void> {
//     // Your custom logic
//   }
//   async getAll(): Promise<Record<string, ShareEntry>> {
//     // Your custom logic
//   }
//   async has(shareId: string): Promise<boolean> {
//     // Your custom logic
//   }
// }
// shareService.setStorageAdapter(new CustomStorageAdapter());
