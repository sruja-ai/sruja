# Code Organization Guide

This document explains the feature-based code organization structure for the website application.

## 📁 Directory Structure

```
src/
├── features/              # Feature modules
│   ├── viewer/           # Interactive DSL viewer
│   ├── content/          # Content display & navigation
│   ├── challenges/       # Coding challenges
│   ├── quizzes/          # Interactive quizzes
│   ├── courses/          # Course content
│   ├── tutorials/        # Tutorial content
│   ├── blog/             # Blog posts
│   ├── playground/       # Code playground
│   ├── studio/           # Design studio
│   ├── search/           # Search functionality
│   ├── home/             # Home page
│   └── documentation/    # Documentation
│
└── shared/               # Shared code across features
    ├── components/       # Reusable UI components
    │   ├── ui/          # UI primitives (TagList, EmptyState, etc.)
    │   ├── layout/      # Layout components (Navbar)
    │   └── content/     # Content components (ContentMeta)
    ├── hooks/           # Reusable React hooks
    ├── utils/           # Utility functions
    ├── lib/             # Business logic
    ├── constants/       # Shared constants
    └── __tests__/       # Shared test utilities
```

## 🎯 Features

Each feature is self-contained with:

- **Components**: Feature-specific UI components
- **Hooks**: Feature-specific React hooks
- **Utils**: Feature-specific utility functions
- **__tests__**: Feature-specific tests

### Example Feature Structure

```
features/viewer/
├── components/
│   ├── InteractiveViewer.tsx
│   ├── ViewerApp.tsx
│   ├── ErrorModal.tsx
│   └── TopNavBar.tsx
├── hooks/
│   ├── useDslState.ts
│   ├── useDslParser.ts
│   └── useWasm.ts
├── utils/
│   ├── storage.ts
│   ├── downloads.ts
│   └── urlState.ts
├── constants.ts
├── types.ts
└── styles.ts
```

## 🔗 Shared Code

### Shared Components

Located in `shared/components/`:

- **UI Components**: `TagList`, `EmptyState`, `SrujaLoader`, `CodeBlockActions`
- **Layout Components**: `Navbar`, `ThemeWrapper`
- **Content Components**: `ContentMeta`, `ContentHeader`

### Shared Hooks

Located in `shared/hooks/`:

- **useLocalStorage**: React hook for localStorage with JSON serialization
- **useExpansion**: Hook for managing expand/collapse state

### Shared Utilities

Located in `shared/utils/`:

- **storage.ts**: Generic localStorage helpers
- **date.ts**: Date formatting utilities
- **analytics.ts**: Event tracking utilities
- **errors.ts**: Error formatting utilities

### Shared Constants

Located in `shared/constants/`:

- **storage.ts**: Storage keys used across the application

## 📝 Import Paths

Use path aliases for clean imports:

```typescript
// Feature imports
import InteractiveViewer from '@/features/viewer/components/InteractiveViewer';
import ChallengeRunner from '@/features/challenges/components/ChallengeRunner';

// Shared imports
import { TagList } from '@/shared/components/ui/TagList';
import { useLocalStorage } from '@/shared/hooks/useLocalStorage';
import { formatDate } from '@/shared/utils/date';
import { STORAGE_KEYS } from '@/shared/constants/storage';
```

## 🧪 Testing

Tests are co-located with source files:

```
src/
├── shared/
│   ├── utils/
│   │   ├── storage.ts
│   │   └── storage.test.ts    # Co-located test
│   └── hooks/
│       ├── useLocalStorage.ts
│       └── useLocalStorage.test.ts
└── features/
    └── viewer/
        ├── components/
        │   ├── InteractiveViewer.tsx
        │   └── InteractiveViewer.test.tsx
        └── __tests__/
            └── viewer.integration.test.ts
```

### Running Tests

```bash
# Unit tests
npm run test

# Watch mode
npm run test:watch

# Coverage
npm run test:coverage

# E2E tests
npm run test:e2e
```

See [TESTING_SETUP.md](./TESTING_SETUP.md) for detailed testing guide.

## 🚀 Adding New Features

1. Create feature directory: `src/features/your-feature/`
2. Add components: `components/YourComponent.tsx`
3. Add hooks if needed: `hooks/useYourHook.ts`
4. Add utilities if needed: `utils/yourUtils.ts`
5. Write tests: `components/YourComponent.test.tsx`

## 📦 Sharing Code Between Features

If code is used by 2+ features:

1. **Components** → Move to `shared/components/`
2. **Hooks** → Move to `shared/hooks/`
3. **Utils** → Move to `shared/utils/`
4. **Constants** → Move to `shared/constants/`

### When to Extract to Shared

✅ **Extract when:**
- Used by 2+ features
- Generic/reusable logic
- UI patterns (buttons, modals, etc.)

❌ **Keep in feature when:**
- Feature-specific logic
- Tightly coupled to feature
- Not reusable elsewhere

## 🔍 Finding Code

- **Feature code**: `src/features/[feature-name]/`
- **Shared utilities**: `src/shared/utils/`
- **Shared components**: `src/shared/components/`
- **Shared hooks**: `src/shared/hooks/`

## 📚 Additional Resources

- [Code Organization Proposal](../archive/CODE_ORGANIZATION_PROPOSAL.md) - Detailed proposal (archived)
- [Migration Status](../archive/MIGRATION_STATUS.md) - Migration progress (archived)
- [Testing Setup](./TESTING_SETUP.md) - Testing guide
