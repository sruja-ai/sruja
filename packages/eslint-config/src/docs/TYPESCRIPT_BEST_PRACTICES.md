# TypeScript Best Practices - FAANG Quality Guide

## Table of Contents

1. [Core Principles](#core-principles)
2. [Type Safety and Strictness](#type-safety-and-strictness)
3. [Code Organization](#code-organization)
4. [Naming Conventions](#naming-conventions)
5. [Error Handling](#error-handling)
6. [Performance Optimization](#performance-optimization)
7. [React + TypeScript Patterns](#react--typescript-patterns)
8. [Async/Await Patterns](#asyncawait-patterns)
9. [Generic Programming](#generic-programming)
10. [Type Guards and Narrowing](#type-guards-and-narrowing)
11. [API Design](#api-design)
12. [Documentation Standards](#documentation-standards)
13. [Common Anti-patterns](#common-anti-patterns)
14. [Code Review Checklist](#code-review-checklist)
15. [Migration Strategies](#migration-strategies)

---

## Core Principles

### Philosophy

At Sruja, we believe TypeScript is a **productivity tool**, not just a type checker. Our approach emphasizes:

1. **Type-Driven Development**: Types are first-class citizens that guide implementation
2. **Eliminate Runtime Errors**: If it compiles, it should work
3. **Self-Documenting Code**: Types serve as live documentation
4. **Refactor with Confidence**: Strong types enable fearless refactoring
5. **Developer Experience**: Types should make code easier to read and understand

### The Three Laws of TypeScript

1. **Never lie to the compiler** - Avoid `as`, `@ts-ignore`, `@ts-expect-error`
2. **Make impossible states unrepresentable** - Use types to prevent bugs
3. **Prefer compile-time checks over runtime checks** - Let TypeScript catch errors

---

## Type Safety and Strictness

### Compiler Configuration

**Always enable strict mode** in `tsconfig.json`:

```json
{
  "compilerOptions": {
    "strict": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictBindCallApply": true,
    "strictPropertyInitialization": true,
    "noImplicitAny": true,
    "noImplicitThis": true,
    "alwaysStrict": true,
    "noImplicitReturns": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "noPropertyAccessFromIndexSignature": true,
    "exactOptionalPropertyTypes": true
  }
}
```

### Eliminate `any`

**Never use `any`**. Use proper types or `unknown` when truly unknown:

```typescript
// BAD
function processData(data: any): any {
  return data.value;
}

// GOOD
function processData<T extends { value: unknown }>(data: T): unknown {
  return data.value;
}

// GOOD (when truly unknown)
function parseInput(input: unknown): Result<DataType, Error> {
  if (typeof input === 'object' && input !== null) {
    // Narrow type
  }
  return err(new Error('Invalid input'));
}
```

### Use `unknown` for Input Validation

```typescript
function validateConfig(config: unknown): Config {
  if (typeof config !== 'object' || config === null) {
    throw new Error('Config must be an object');
  }
  
  if (!('apiKey' in config) || typeof config.apiKey !== 'string') {
    throw new Error('Config.apiKey must be a string');
  }
  
  return {
    apiKey: config.apiKey,
    // ... other validated fields
  };
}
```

### Prefer `readonly` for Immutability

```typescript
// BAD
interface User {
  id: string;
  name: string;
  roles: string[];
}

// GOOD
interface User {
  readonly id: string;
  readonly name: string;
  readonly roles: readonly string[];
}

// GOOD (for collections)
function getActiveUsers(users: readonly User[]): User[] {
  return users.filter(user => user.isActive);
}
```

### Use Branded Types for Domain Values

```typescript
// Primitive types don't prevent mixing up similar values
type UserId = string & { readonly __brand: unique symbol };
type Email = string & { readonly __brand: unique symbol };
type Timestamp = number & { readonly __brand: unique symbol };

// Factory functions
const UserId = (id: string): UserId => id as UserId;
const Email = (email: string): Email => {
  if (!isValidEmail(email)) {
    throw new Error('Invalid email');
  }
  return email as Email;
};

// Usage - type-safe
function getUserById(id: UserId): User { /* ... */ }
function sendEmail(to: Email, subject: string): void { /* ... */ }

// Compile error - prevents mixing
// getUserById(userEmail); // ❌ Type error
```

### Discriminated Unions for State Machines

```typescript
// BAD
interface RequestState {
  loading: boolean;
  data?: Data;
  error?: Error;
}

// GOOD - Impossible states eliminated
type RequestState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: Data }
  | { status: 'error'; error: Error };

function renderRequest(state: RequestState): ReactNode {
  switch (state.status) {
    case 'idle':
      return <IdleState />;
    case 'loading':
      return <LoadingSpinner />;
    case 'success':
      return <DataDisplay data={state.data} />; // TypeScript knows data exists
    case 'error':
      return <ErrorMessage error={state.error} />; // TypeScript knows error exists
  }
}
```

### Template Literal Types for APIs

```typescript
// Type-safe API endpoint construction
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

type ApiEndpoint = `/api/${string}`;

function request<T>(
  method: HttpMethod,
  endpoint: ApiEndpoint,
  data?: unknown
): Promise<T> {
  // Implementation
  return fetch(endpoint, { method, body: JSON.stringify(data) }).then(r => r.json());
}

// Usage - compile-time validation
request<User[]>('GET', '/api/users');
request<User>('POST', '/api/users', { name: 'John' });
// request('GET', '/invalid/endpoint'); // ❌ Type error

// Enhanced: Strict endpoint typing
type ApiEndpoints = {
  '/api/users': { GET: User[]; POST: User; PUT: User };
  '/api/users/:id': { GET: User; DELETE: void };
};

type ApiCall<E extends keyof ApiEndpoints, M extends keyof ApiEndpoints[E]> = {
  endpoint: E;
  method: M;
  data?: ApiEndpoints[E][M] extends { POST: infer P } ? P : never;
};

// API client with type safety
class ApiClient {
  async call<E extends keyof ApiEndpoints, M extends keyof ApiEndpoints[E]>(
    endpoint: E,
    method: M,
    data?: ApiCall<E, M>['data']
  ): Promise<ApiEndpoints[E][M] extends { GET: infer R } ? R : unknown> {
    // Implementation
    return fetch(endpoint.toString(), {
      method: method as string,
      body: JSON.stringify(data)
    }).then(r => r.json());
  }
}
```

---

## Code Organization

### File Structure

Follow a consistent, logical structure:

```
src/
├── components/          # React components
│   ├── Button/
│   │   ├── Button.tsx
│   │   ├── Button.test.tsx
│   │   ├── Button.stories.tsx
│   │   └── index.ts
│   └── Form/
│       ├── Form.tsx
│       ├── Form.test.tsx
│       ├── types.ts
│       └── index.ts
├── hooks/               # React hooks
│   ├── useAuth.ts
│   ├── useAuth.test.ts
│   └── index.ts
├── services/            # External service interfaces
│   ├── api.ts
│   ├── api.test.ts
│   └── index.ts
├── utils/               # Pure utility functions
│   ├── date.ts
│   ├── date.test.ts
│   └── index.ts
├── types/               # Shared types
│   ├── user.ts
│   ├── api.ts
│   └── index.ts
├── constants/           # Constants and enums
│   ├── routes.ts
│   └── index.ts
├── config/              # Configuration
│   ├── env.ts
│   └── index.ts
└── index.ts             # Public API
```

### Barrel Exports (index.ts)

Use barrel files to clean up imports:

```typescript
// types/index.ts
export * from './user';
export * from './api';

// Instead of:
import { User, ApiError } from '../../../types/user';
import { ApiResponse } from '../../../types/api';

// Use:
import { User, ApiError, ApiResponse } from '@/types';
```

### Type Files Grouping

```typescript
// types/user.ts
export type UserId = string & { readonly __brand: unique symbol };
export type Email = string & { readonly __brand: unique symbol };

export interface User {
  readonly id: UserId;
  readonly email: Email;
  readonly name: string;
  readonly createdAt: Date;
  readonly updatedAt: Date;
}

export type UserCreateInput = Omit<User, 'id' | 'createdAt' | 'updatedAt'>;
export type UserUpdateInput = Partial<UserCreateInput>;

// types/api.ts
export type ApiResponse<T> = 
  | { success: true; data: T }
  | { success: false; error: ApiError };

export interface ApiError {
  readonly code: string;
  readonly message: string;
  readonly details?: Record<string, unknown>;
}

export type PaginatedResponse<T> = {
  readonly data: readonly T[];
  readonly total: number;
  readonly page: number;
  readonly pageSize: number;
};
```

---

## Naming Conventions

### Types and Interfaces

```typescript
// Interfaces for object shapes
interface User { /* ... */ }
interface ApiClient { /* ... */ }

// Types for unions, primitives, complex types
type UserId = string & { readonly __brand: unique symbol };
type RequestState = { status: 'loading' } | { status: 'success'; data: Data };

// Enum-like types
type Theme = 'light' | 'dark' | 'system';
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

// Generic type parameters - single uppercase
function map<T, U>(array: readonly T[], fn: (item: T) => U): U[] { /* ... */ }

// Component props interfaces
interface ButtonProps {
  readonly variant: 'primary' | 'secondary';
  readonly size: 'small' | 'medium' | 'large';
  readonly onClick: () => void;
  readonly children: ReactNode;
}
```

### Functions and Methods

```typescript
// camelCase for functions
function getUserById(id: UserId): Promise<User> { /* ... */ }
function formatDate(date: Date): string { /* ... */ }

// Prefix patterns
const isUser = (value: unknown): value is User => { /* ... */ }; // Type guard
const hasPermission = (user: User, permission: string): boolean => { /* ... */ };
const canEdit = (user: User, resource: Resource): boolean => { /* ... */ };

// Async functions - use Promise return type
async function fetchUser(id: UserId): Promise<User> { /* ... */ }

// Boolean functions - is/has/can
function isValidEmail(email: string): boolean { /* ... */ }
function hasRole(user: User, role: string): boolean { /* ... */ }
function canDelete(user: User, resource: Resource): boolean { /* ... */ }

// Conversion functions - to/from
function toString(value: unknown): string { /* ... */ }
function fromDateString(date: string): Date { /* ... */ }

// Builder patterns
function createUser(input: UserCreateInput): User { /* ... */ }
function buildQuery(filters: QueryFilters): string { /* ... */ }
```

### Variables and Properties

```typescript
// camelCase
const userId: UserId = ...;
const userName: string = ...;
const isActive: boolean = ...;

// Booleans - is/has/can prefix
const isLoading = false;
const hasError = false;
const canEdit = true;

// Collections - plural
const users: User[] = ...;
const userMap: Map<UserId, User> = ...;
const userById: Record<UserId, User> = ...;

// Constants - UPPER_SNAKE_CASE
const API_BASE_URL = 'https://api.example.com';
const MAX_RETRY_COUNT = 3;
const DEFAULT_PAGE_SIZE = 20;

// Private class properties - # prefix
class ApiClient {
  #apiKey: string;
  #cache: Map<string, unknown>;
}
```

### Type Guards

```typescript
// is prefix for type guards
function isUser(value: unknown): value is User { /* ... */ }
function isError(value: unknown): value is Error { /* ... */ }
function isApiResponse<T>(value: unknown): value is ApiResponse<T> { /* ... */ }

// Usage
if (isUser(data)) {
  // TypeScript knows data is User
  console.log(data.name);
}
```

---

## Error Handling

### Result Type Pattern

Never throw for expected errors. Use Result type:

```typescript
// Result type implementation
export type Result<T, E = Error> = Ok<T> | Err<E>;

export interface Ok<T> {
  readonly _tag: 'ok';
  readonly value: T;
}

export interface Err<E> {
  readonly _tag: 'err';
  readonly error: E;
}

// Constructors
export const ok = <T>(value: T): Ok<T> => ({ _tag: 'ok', value });
export const err = <E>(error: E): Err<E> => ({ _tag: 'err', error });

// Utility functions
export const isOk = <T, E>(result: Result<T, E>): result is Ok<T> =>
  result._tag === 'ok';

export const isErr = <T, E>(result: Result<T, E>): result is Err<E> =>
  result._tag === 'err';

export const map = <T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => U
): Result<U, E> =>
  isOk(result) ? ok(fn(result.value)) : result;

export const andThen = <T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => Result<U, E>
): Result<U, E> =>
  isOk(result) ? fn(result.value) : result;

export const unwrap = <T, E>(result: Result<T, E>): T => {
  if (isOk(result)) return result.value;
  throw result.error;
};

export const unwrapOr = <T, E>(result: Result<T, E>, defaultValue: T): T =>
  isOk(result) ? result.value : defaultValue;

// Usage
function fetchUser(id: UserId): Promise<Result<User, ApiError>> {
  return fetch(`/api/users/${id}`)
    .then(res => res.json())
    .then(data => ok(data))
    .catch(error => err({ code: 'FETCH_ERROR', message: error.message }));
}

// Handling
async function displayUser(id: UserId): Promise<void> {
  const result = await fetchUser(id);
  
  if (isOk(result)) {
    console.log(`User: ${result.value.name}`);
  } else {
    console.error(`Error: ${result.error.message}`);
  }
}

// Chaining with map/andThen
async function getUserEmail(id: UserId): Promise<Result<string, ApiError>> {
  return fetchUser(id)
    .then(result => map(result, user => user.email))
    .then(result => andThen(result, email => {
      if (!isValidEmail(email)) {
        return err({ code: 'INVALID_EMAIL', message: 'Invalid email format' });
      }
      return ok(email);
    }));
}
```

### Custom Error Types

```typescript
// Base error class
export class SrujaError extends Error {
  readonly code: string;
  readonly details?: Record<string, unknown>;
  
  constructor(message: string, code: string, details?: Record<string, unknown>) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.details = details;
    
    // Maintains proper stack trace for where our error was thrown
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
  
  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      details: this.details,
      stack: this.stack,
    };
  }
}

// Domain-specific errors
export class ValidationError extends SrujaError {
  constructor(message: string, field: string) {
    super(message, 'VALIDATION_ERROR', { field });
  }
}

export class AuthenticationError extends SrujaError {
  constructor(message: string) {
    super(message, 'AUTHENTICATION_ERROR');
  }
}

export class AuthorizationError extends SrujaError {
  constructor(message: string, resource?: string) {
    super(message, 'AUTHORIZATION_ERROR', { resource });
  }
}

export class NetworkError extends SrujaError {
  constructor(message: string, url?: string) {
    super(message, 'NETWORK_ERROR', { url });
  }
}

// Error type guard
function isSrujaError(error: unknown): error is SrujaError {
  return error instanceof SrujaError;
}

// Safe error parsing
function getErrorMessage(error: unknown): string {
  if (isSrujaError(error)) return error.message;
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Unknown error occurred';
}
```

### Error Boundaries (React)

```typescript
// Error boundary with typed error
interface ErrorBoundaryProps {
  readonly children: ReactNode;
  readonly fallback?: ReactNode;
  readonly onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  readonly hasError: boolean;
  readonly error?: Error;
}

export class ErrorBoundary extends React.Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    this.props.onError?.(error, errorInfo);
    
    // Log error to monitoring service
    logError(error, { componentStack: errorInfo.componentStack });
  }

  render(): ReactNode {
    if (this.state.hasError) {
      return this.props.fallback || <ErrorFallback error={this.state.error} />;
    }
    
    return this.props.children;
  }
}

// Usage
function App(): ReactNode {
  return (
    <ErrorBoundary
      fallback={<div>Something went wrong</div>}
      onError={(error, errorInfo) => {
        console.error('Error caught by boundary:', error, errorInfo);
      }}
    >
      <MainContent />
    </ErrorBoundary>
  );
}
```

---

## Performance Optimization

### Memoization with Generic Types

```typescript
// Type-safe memoization
function memoize<T extends (...args: readonly unknown[]) => unknown>(
  fn: T,
  keyGenerator?: (...args: Parameters<T>) => string
): T & { readonly cache: Map<string, ReturnType<T>> } {
  const cache = new Map<string, ReturnType<T>>();
  
  const memoized = ((...args: Parameters<T>) => {
    const key = keyGenerator
      ? keyGenerator(...args)
      : JSON.stringify(args);
    
    if (cache.has(key)) {
      return cache.get(key)!;
    }
    
    const result = fn(...args);
    cache.set(key, result);
    return result;
  }) as T & { readonly cache: Map<string, ReturnType<T>> };
  
  memoized.cache = cache;
  
  return memoized;
}

// Usage
const expensiveCalculation = (input: string): number => {
  // Expensive computation
  return input.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
};

const memoizedCalculation = memoize(expensiveCalculation);

// Weak memoization for objects
function weakMemoize<T extends object, U>(
  fn: (obj: T) => U
): (obj: T) => U {
  const cache = new WeakMap<T, U>();
  
  return (obj: T) => {
    if (cache.has(obj)) {
      return cache.get(obj)!;
    }
    
    const result = fn(obj);
    cache.set(obj, result);
    return result;
  };
}
```

### React Performance with TypeScript

```typescript
// Properly typed React.memo
interface UserProps {
  readonly user: User;
  readonly onClick: (userId: UserId) => void;
}

export const UserCard = React.memo<UserProps>(
  ({ user, onClick }) => (
    <div onClick={() => onClick(user.id)}>
      <h3>{user.name}</h3>
      <p>{user.email}</p>
    </div>
  ),
  (prevProps, nextProps) => {
    return (
      prevProps.user.id === nextProps.user.id &&
      prevProps.user.name === nextProps.user.name &&
      prevProps.user.email === nextProps.user.email &&
      prevProps.onClick === nextProps.onClick
    );
  }
);

// Generic useDeepCompareMemo
function useDeepCompareMemo<T>(value: T): T {
  const ref = useRef<T>();
  const signalRef = useRef<number>(0);
  
  if (!deepEqual(ref.current, value)) {
    ref.current = value;
    signalRef.current += 1;
  }
  
  return useMemo(() => ref.current!, [signalRef.current]);
}

// Generic useDeepCompareCallback
function useDeepCompareCallback<T extends (...args: never[]) => unknown>(
  callback: T,
  deps: readonly unknown[]
): T {
  return useCallback(callback, useDeepCompareMemo(deps));
}

// Optimized list rendering
interface VirtualListProps<T> {
  readonly items: readonly T[];
  readonly renderItem: (item: T, index: number) => ReactNode;
  readonly itemHeight: number;
  readonly containerHeight: number;
}

export function VirtualList<T>({
  items,
  renderItem,
  itemHeight,
  containerHeight
}: VirtualListProps<T>): ReactNode {
  const [scrollTop, setScrollTop] = useState(0);
  
  const visibleStart = Math.floor(scrollTop / itemHeight);
  const visibleEnd = Math.min(
    visibleStart + Math.ceil(containerHeight / itemHeight),
    items.length
  );
  
  const visibleItems = items.slice(visibleStart, visibleEnd);
  
  return (
    <div
      style={{ height: containerHeight, overflowY: 'auto' }}
      onScroll={(e) => setScrollTop((e.target as HTMLElement).scrollTop)}
    >
      <div style={{ height: items.length * itemHeight, position: 'relative' }}>
        {visibleItems.map((item, index) => (
          <div
            key={visibleStart + index}
            style={{
              position: 'absolute',
              top: (visibleStart + index) * itemHeight,
              height: itemHeight,
            }}
          >
            {renderItem(item, visibleStart + index)}
          </div>
        ))}
      </div>
    </div>
  );
}
```

### Debounce and Throttle with Types

```typescript
// Type-safe debounce
function debounce<T extends (...args: never[]) => void>(
  fn: T,
  delay: number
): T & { readonly cancel: () => void } {
  let timeoutId: NodeJS.Timeout | null = null;
  
  const debounced = ((...args: Parameters<T>) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    
    timeoutId = setTimeout(() => {
      fn(...args);
    }, delay);
  }) as T & { readonly cancel: () => void };
  
  debounced.cancel = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
  };
  
  return debounced;
}

// Type-safe throttle
function throttle<T extends (...args: never[]) => void>(
  fn: T,
  limit: number
): T {
  let inThrottle: boolean = false;
  
  return ((...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
      }, limit);
    }
  }) as T;
}

// Usage
const handleResize = debounce((event: UIEvent) => {
  console.log('Window resized:', event);
}, 300);

const handleScroll = throttle((event: Event) => {
  console.log('Scrolled:', event);
}, 100);
```

---

## React + TypeScript Patterns

### Component Props

```typescript
// Use readonly for props
interface ButtonProps {
  readonly variant: 'primary' | 'secondary' | 'danger';
  readonly size: 'small' | 'medium' | 'large';
  readonly disabled?: boolean;
  readonly loading?: boolean;
  readonly onClick: () => void;
  readonly children: ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  variant,
  size,
  disabled = false,
  loading = false,
  onClick,
  children
}) => {
  return (
    <button
      disabled={disabled || loading}
      onClick={onClick}
      className={`btn btn-${variant} btn-${size}`}
    >
      {loading ? 'Loading...' : children}
    </button>
  );
};

// Generic container components
interface ContainerProps<T> {
  readonly items: readonly T[];
  readonly renderItem: (item: T, index: number) => ReactNode;
  readonly keyExtractor: (item: T) => string | number;
}

export function Container<T>({ items, renderItem, keyExtractor }: ContainerProps<T>): ReactNode {
  return (
    <div>
      {items.map((item, index) => (
        <div key={keyExtractor(item)}>
          {renderItem(item, index)}
        </div>
      ))}
    </div>
  );
}

// Usage
interface User {
  readonly id: string;
  readonly name: string;
}

<Container<User>
  items={users}
  renderItem={(user) => <div>{user.name}</div>}
  keyExtractor={(user) => user.id}
/>
```

### Custom Hooks

```typescript
// Generic useAsync hook
interface AsyncState<T> {
  readonly data: T | null;
  readonly error: Error | null;
  readonly isLoading: boolean;
}

type AsyncAction<T> = 
  | { readonly type: 'fetch' }
  | { readonly type: 'success'; readonly payload: T }
  | { readonly type: 'error'; readonly payload: Error };

function asyncReducer<T>(state: AsyncState<T>, action: AsyncAction<T>): AsyncState<T> {
  switch (action.type) {
    case 'fetch':
      return { data: state.data, error: null, isLoading: true };
    case 'success':
      return { data: action.payload, error: null, isLoading: false };
    case 'error':
      return { data: null, error: action.payload, isLoading: false };
  }
}

function useAsync<T>(asyncFunction: () => Promise<T>, immediate = true): AsyncState<T> & { readonly execute: () => Promise<void> } {
  const [state, dispatch] = React.useReducer(asyncReducer<T>, {
    data: null,
    error: null,
    isLoading: immediate,
  });

  const execute = React.useCallback(async () => {
    dispatch({ type: 'fetch' });
    try {
      const result = await asyncFunction();
      dispatch({ type: 'success', payload: result });
    } catch (error) {
      dispatch({ type: 'error', payload: error as Error });
    }
  }, [asyncFunction]);

  React.useEffect(() => {
    if (immediate) {
      execute();
    }
  }, [execute, immediate]);

  return { ...state, execute };
}

// Usage
const { data, error, isLoading, execute } = useAsync(() => fetchUser(userId));
```

### Form Handling

```typescript
// Type-safe form state
interface FormState<T> {
  readonly values: T;
  readonly errors: Partial<Record<keyof T, string>>;
  readonly touched: Partial<Record<keyof T, boolean>>;
  readonly isDirty: boolean;
  readonly isValid: boolean;
}

type FormAction<T> =
  | { readonly type: 'SET_VALUE'; readonly field: keyof T; readonly value: T[keyof T] }
  | { readonly type: 'SET_ERROR'; readonly field: keyof T; readonly error: string }
  | { readonly type: 'SET_TOUCHED'; readonly field: keyof T; readonly touched: boolean }
  | { readonly type: 'RESET' };

function formReducer<T>(state: FormState<T>, action: FormAction<T>): FormState<T> {
  switch (action.type) {
    case 'SET_VALUE':
      return {
        ...state,
        values: { ...state.values, [action.field]: action.value },
        isDirty: true,
      };
    case 'SET_ERROR':
      return {
        ...state,
        errors: { ...state.errors, [action.field]: action.error },
      };
    case 'SET_TOUCHED':
      return {
        ...state,
        touched: { ...state.touched, [action.field]: action.touched },
      };
    case 'RESET':
      return {
        values: state.values,
        errors: {},
        touched: {},
        isDirty: false,
        isValid: true,
      };
  }
}

interface UseFormOptions<T> {
  readonly initialValues: T;
  readonly validate?: (values: T) => Partial<Record<keyof T, string>>;
  readonly onSubmit: (values: T) => void | Promise<void>;
}

function useForm<T extends Record<string, unknown>>({
  initialValues,
  validate,
  onSubmit
}: UseFormOptions<T>): FormState<T> & {
  readonly setValue: (field: keyof T, value: T[keyof T]) => void;
  readonly setError: (field: keyof T, error: string) => void;
  readonly setTouched: (field: keyof T, touched: boolean) => void;
  readonly handleSubmit: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  readonly reset: () => void;
} {
  const [state, dispatch] = React.useReducer(formReducer<T>, {
    values: initialValues,
    errors: {},
    touched: {},
    isDirty: false,
    isValid: true,
  });

  const setValue = React.useCallback(
    (field: keyof T, value: T[keyof T]) => {
      dispatch({ type: 'SET_VALUE', field, value });
    },
    []
  );

  const setError = React.useCallback(
    (field: keyof T, error: string) => {
      dispatch({ type: 'SET_ERROR', field, error });
    },
    []
  );

  const setTouched = React.useCallback(
    (field: keyof T, touched: boolean) => {
      dispatch({ type: 'SET_TOUCHED', field, touched });
    },
    []
  );

  const handleSubmit = React.useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();

      // Validate
      if (validate) {
        const errors = validate(state.values);
        Object.entries(errors).forEach(([field, error]) => {
          setError(field as keyof T, error!);
        });
        
        if (Object.keys(errors).length > 0) {
          return;
        }
      }

      // Submit
      await onSubmit(state.values);
    },
    [state.values, validate, onSubmit, setError]
  );

  const reset = React.useCallback(() => {
    dispatch({ type: 'RESET' });
  }, []);

  return {
    ...state,
    setValue,
    setError,
    setTouched,
    handleSubmit,
    reset,
  };
}
```

---

## Async/Await Patterns

### Async Type Safety

```typescript
// Always type async function returns
async function fetchData(id: string): Promise<Result<Data, Error>> {
  try {
    const response = await fetch(`/api/data/${id}`);
    const data = await response.json();
    return ok(data);
  } catch (error) {
    return err(error instanceof Error ? error : new Error(String(error)));
  }
}

// Parallel async operations with proper typing
async function fetchMultipleData(
  ids: readonly string[]
): Promise<Map<string, Result<Data, Error>>> {
  const results = await Promise.allSettled(
    ids.map(id => fetchData(id))
  );
  
  const resultMap = new Map<string, Result<Data, Error>>();
  
  results.forEach((result, index) => {
    const id = ids[index];
    if (result.status === 'fulfilled') {
      resultMap.set(id, result.value);
    } else {
      resultMap.set(id, err(result.reason));
    }
  });
  
  return resultMap;
}

// Generic retry with exponential backoff
async function retry<T>(
  fn: () => Promise<T>,
  options: {
    readonly maxAttempts?: number;
    readonly delay?: number;
    readonly backoff?: number;
    readonly shouldRetry?: (error: unknown) => boolean;
  } = {}
): Promise<T> {
  const {
    maxAttempts = 3,
    delay = 1000,
    backoff = 2,
    shouldRetry = () => true,
  } = options;
  
  let lastError: unknown;
  
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      
      if (!shouldRetry(error) || attempt === maxAttempts) {
        throw error;
      }
      
      const waitTime = delay * Math.pow(backoff, attempt - 1);
      await new Promise(resolve => setTimeout(resolve, waitTime));
    }
  }
  
  throw lastError;
}

// Usage
const data = await retry(
  () => fetchData('123'),
  {
    maxAttempts: 5,
    delay: 1000,
    shouldRetry: (error) => {
      if (error instanceof NetworkError) {
        return error.code !== '404';
      }
      return false;
    },
  }
);
```

### Async Generators

```typescript
// Type-safe async generator
async function* paginateResults<T>(
  fetchPage: (page: number) => Promise<readonly T[]>,
  pageSize: number
): AsyncGenerator<readonly T[], void, unknown> {
  let page = 1;
  let hasMore = true;
  
  while (hasMore) {
    const results = await fetchPage(page);
    yield results;
    
    if (results.length < pageSize) {
      hasMore = false;
    } else {
      page++;
    }
  }
}

// Usage
for await (const page of paginateResults(fetchUsersPage, 20)) {
  console.log('Page of users:', page);
}

// Generic async iterator with cancellation
function createAsyncIterator<T>(
  source: AsyncIterable<T>,
  signal: AbortSignal
): AsyncIterable<T> {
  const asyncIterator = source[Symbol.asyncIterator]();
  
  return {
    [Symbol.asyncIterator]() {
      return {
        async next() {
          if (signal.aborted) {
            return { done: true, value: undefined };
          }
          
          return asyncIterator.next();
        },
        async return() {
          return asyncIterator.return?.();
        },
      };
    },
  };
}
```

---

## Generic Programming

### Utility Types

```typescript
// Deep readonly
type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

// Deep partial
type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

// Deep required
type DeepRequired<T> = {
  [P in keyof T]-?: T[P] extends object ? DeepRequired<T[P]> : T[P];
};

// Deep nullable
type DeepNullable<T> = {
  [P in keyof T]: T[P] extends object ? DeepNullable<T[P]> : T[P] | null;
};

// Pick by condition
type PickByType<T, U> = {
  [P in keyof T as T[P] extends U ? P : never]: T[P];
};

// Omit by type
type OmitByType<T, U> = {
  [P in keyof T as T[P] extends U ? never : P]: T[P];
};

// Make some properties optional
type PartialSome<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

// Make some properties required
type RequiredSome<T, K extends keyof T> = Omit<T, K> & Required<Pick<T, K>>;

// Brand type
type Branded<T, B> = T & { readonly __brand: B };
type Brand<T, B> = T & { readonly __brand: B };

// Merge two types
type Merge<T, U> = Omit<T, keyof U> & U;

// Override specific properties
type Override<T, U> = Omit<T, keyof U> & U;
```

### Generic Constraints

```typescript
// Ensure object has specific property
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

// Ensure callback returns specific type
function map<T, U>(array: readonly T[], fn: (item: T) => U): U[] {
  return array.map(fn);
}

// Ensure object matches interface
function validateConfig<T extends Record<string, unknown>>(
  config: T,
  schema: { readonly [K in keyof T]?: (value: unknown) => boolean }
): boolean {
  return Object.entries(schema).every(([key, validator]) => {
    const value = config[key as keyof T];
    return validator?.(value) ?? true;
  });
}

// Generic factory with constraints
interface Identifiable {
  readonly id: string;
}

function createRepository<T extends Identifiable>(
  items: readonly T[]
): {
  readonly find: (id: string) => T | undefined;
  readonly filter: (predicate: (item: T) => boolean) => readonly T[];
  readonly all: () => readonly T[];
} {
  return {
    find: (id) => items.find(item => item.id === id),
    filter: (predicate) => items.filter(predicate),
    all: () => items,
  };
}

// Conditional generic types
type ActionType<T, U> = T extends U ? T : never;
type ExtractKeysOfType<T, U> = {
  [K in keyof T]: T[K] extends U ? K : never;
}[keyof T];
```

### Generic Components

```typescript
// Generic list component
interface ListProps<T> {
  readonly items: readonly T[];
  readonly renderItem: (item: T) => ReactNode;
  readonly keyExtractor: (item: T) => string | number;
  readonly emptyMessage?: string;
}

export function List<T>({
  items,
  renderItem,
  keyExtractor,
  emptyMessage = 'No items'
}: ListProps<T>): ReactNode {
  if (items.length === 0) {
    return <div>{emptyMessage}</div>;
  }
  
  return (
    <ul>
      {items.map(item => (
        <li key={keyExtractor(item)}>
          {renderItem(item)}
        </li>
      ))}
    </ul>
  );
}

// Generic table component
interface Column<T> {
  readonly key: string;
  readonly title: string;
  readonly render: (item: T) => ReactNode;
}

interface TableProps<T> {
  readonly data: readonly T[];
  readonly columns: readonly Column<T>[];
  readonly onRowClick?: (item: T) => void;
}

export function Table<T>({ data, columns, onRowClick }: TableProps<T>): ReactNode {
  return (
    <table>
      <thead>
        <tr>
          {columns.map(column => (
            <th key={column.key}>{column.title}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {data.map((row, index) => (
          <tr
            key={index}
            onClick={() => onRowClick?.(row)}
            style={{ cursor: onRowClick ? 'pointer' : 'default' }}
          >
            {columns.map(column => (
              <td key={column.key}>{column.render(row)}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

---

## Type Guards and Narrowing

### Type Guards

```typescript
// Primitive type guards
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !isNaN(value);
}

function isBoolean(value: unknown): value is boolean {
  return typeof value === 'boolean';
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

// Generic type guard for discriminated unions
type Discriminator<T, K extends keyof T> = {
  [P in K]: T[P];
};

function hasDiscriminator<T, K extends keyof T>(
  obj: unknown,
  key: K,
  value: T[K]
): obj is Discriminator<T, K> {
  return isObject(obj) && key in obj && obj[key] === value;
}

// Example usage
type ApiResponse<T> =
  | { readonly status: 'success'; readonly data: T }
  | { readonly status: 'error'; readonly error: string };

function isSuccessResponse<T>(response: ApiResponse<T>): response is { readonly status: 'success'; readonly data: T } {
  return response.status === 'success';
}

// Property existence type guard
function hasProperty<T extends object, K extends PropertyKey>(
  obj: T,
  prop: K
): obj is T & Record<K, unknown> {
  return prop in obj;
}

// Array element type guard
function isArrayOf<T>(
  value: unknown,
  guard: (item: unknown) => item is T
): value is T[] {
  if (!Array.isArray(value)) {
    return false;
  }
  return value.every(item => guard(item));
}
```

### Type Narrowing

```typescript
// Using type predicates
function processValue(value: string | number | null): void {
  if (value === null) {
    console.log('Value is null');
    return;
  }
  
  if (typeof value === 'string') {
    // TypeScript knows value is string
    console.log(value.toUpperCase());
  } else {
    // TypeScript knows value is number
    console.log(value.toFixed(2));
  }
}

// Using discriminated unions
type User = 
  | { readonly type: 'guest'; readonly id: string }
  | { readonly type: 'registered'; readonly id: string; readonly email: string };

function getUserEmail(user: User): string {
  switch (user.type) {
    case 'guest':
      return 'guest@example.com';
    case 'registered':
      // TypeScript knows email exists
      return user.email;
  }
}

// Using custom type guards
interface Error {
  readonly message: string;
}

interface NetworkError extends Error {
  readonly type: 'network';
  readonly status: number;
}

interface ValidationError extends Error {
  readonly type: 'validation';
  readonly field: string;
}

type AppError = NetworkError | ValidationError;

function isNetworkError(error: AppError): error is NetworkError {
  return error.type === 'network';
}

function handleError(error: AppError): string {
  if (isNetworkError(error)) {
    return `Network error: ${error.status}`;
  }
  // TypeScript knows it's ValidationError
  return `Validation error: ${error.field} - ${error.message}`;
}

// Using assertion functions
function assertIsDefined<T>(value: T | null | undefined): asserts value is T {
  if (value === null || value === undefined) {
    throw new Error(`Value is not defined`);
  }
}

function processUser(user: User | null): void {
  assertIsDefined(user);
  // TypeScript knows user is defined
  console.log(user.id);
}

function assertIsString(value: unknown): asserts value is string {
  if (typeof value !== 'string') {
    throw new Error(`Value is not a string`);
  }
}

function parseConfig(config: unknown): Config {
  if (!isObject(config)) {
    throw new Error('Config must be an object');
  }
  
  assertIsString(config.apiKey);
  // TypeScript knows config.apiKey is string
  return {
    apiKey: config.apiKey,
    // ...
  };
}
```

---

## API Design

### Function Signatures

```typescript
// Use descriptive parameter names
// BAD
function process(a: string, b: number, c: boolean): void { /* ... */ }

// GOOD
function processRequest(
  url: string,
  timeoutMs: number,
  shouldRetry: boolean
): void { /* ... */ }

// Use consistent parameter order: required first, optional last
// BAD
function createButton(
  disabled?: boolean,
  children: ReactNode,
  onClick: () => void
): ReactNode { /* ... */ }

// GOOD
function createButton(
  children: ReactNode,
  onClick: () => void,
  disabled?: boolean
): ReactNode { /* ... */ }

// Use options object for many parameters
// BAD
function createUser(
  name: string,
  email: string,
  age?: number,
  address?: string,
  phone?: string,
  admin?: boolean
): User { /* ... */ }

// GOOD
interface CreateUserOptions {
  readonly name: string;
  readonly email: string;
  readonly age?: number;
  readonly address?: string;
  readonly phone?: string;
  readonly admin?: boolean;
}

function createUser(options: CreateUserOptions): User { /* ... */ }

// Use callback patterns with proper typing
// BAD
function processData(data: unknown, callback: (result: unknown) => void): void { /* ... */ }

// GOOD
function processData<T, U>(
  data: T,
  callback: (result: U) => void
): void { /* ... */ }

// Use async/await with proper error types
// BAD
async function fetchUser(id: string): Promise<User | null> { /* ... */ }

// GOOD
async function fetchUser(id: string): Promise<Result<User, ApiError>> { /* ... */ }
```

### Interface Design

```typescript
// Prefer interfaces over types for object shapes
// OK
type User = {
  readonly id: string;
  readonly name: string;
};

// BETTER
interface User {
  readonly id: string;
  readonly name: string;
  readonly email: string;
  readonly createdAt: Date;
}

// Use readonly for immutable data
interface User {
  readonly id: string;
  readonly name: string;
  readonly email: string;
  readonly createdAt: Date;
  // No setters - create new object for updates
}

// Use branded types for domain values
type UserId = string & { readonly __brand: unique symbol };
type Email = string & { readonly __brand: unique symbol };

interface User {
  readonly id: UserId;
  readonly email: Email;
}

// Use discriminated unions for related types
type RequestState<T> =
  | { readonly status: 'idle' }
  | { readonly status: 'loading' }
  | { readonly status: 'success'; readonly data: T }
  | { readonly status: 'error'; readonly error: Error };

// Use generic interfaces for reusable components
interface PaginationParams {
  readonly page: number;
  readonly pageSize: number;
}

interface PaginatedResponse<T> {
  readonly data: readonly T[];
  readonly total: number;
  readonly page: number;
  readonly pageSize: number;
}

interface ApiClient {
  readonly getUsers: (params: PaginationParams) => Promise<PaginatedResponse<User>>;
  readonly getUserById: (id: UserId) => Promise<Result<User, ApiError>>;
}
```

---

## Documentation Standards

### TSDoc Comments

```typescript
/**
 * Fetches a user by their unique identifier.
 *
 * @remarks
 * This function retrieves user data from the API. It uses caching
 * to improve performance and includes automatic retry logic for
 * transient failures.
 *
 * @param userId - The unique identifier of the user to fetch
 * @param options - Optional configuration for the request
 * @param options.cache - Whether to use caching (default: true)
 * @param options.timeout - Request timeout in milliseconds (default: 5000)
 * @returns A promise that resolves to the user data
 * @throws {NetworkError} When the network request fails
 * @throws {NotFoundError} When the user doesn't exist
 *
 * @example
 * ```typescript
 * const user = await fetchUser('user-123');
 * console.log(user.name);
 *
 * // With options
 * const user = await fetchUser('user-123', {
 *   cache: false,
 *   timeout: 10000,
 * });
 * ```
 */
async function fetchUser(
  userId: UserId,
  options?: {
    readonly cache?: boolean;
    readonly timeout?: number;
  }
): Promise<User> {
  // Implementation
}

/**
 * Custom error type for API-related errors.
 *
 * @remarks
 * This error type provides additional context about API failures,
 * including the HTTP status code and request URL.
 */
export class ApiError extends Error {
  /**
   * The HTTP status code associated with this error.
   */
  readonly statusCode: number;
  
  /**
   * The URL that was being requested when the error occurred.
   */
  readonly url: string;
  
  /**
   * Additional error details from the API response.
   */
  readonly details?: Record<string, unknown>;
  
  /**
   * Creates a new ApiError instance.
   *
   * @param message - A human-readable error message
   * @param statusCode - The HTTP status code
   * @param url - The request URL
   * @param details - Additional error details
   */
  constructor(
    message: string,
    statusCode: number,
    url: string,
    details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'ApiError';
    this.statusCode = statusCode;
    this.url = url;
    this.details = details;
  }
}
```

### Module Documentation

```typescript
/**
 * @packageDocumentation
 *
 * # User Management Module
 *
 * Provides functionality for managing users in the system, including
 * user creation, retrieval, updates, and deletion operations.
 *
 * ## Core Types
 *
 * - {@link User} - Represents a user in the system
 * - {@link UserId} - Branded type for user identifiers
 * - {@link UserCreateInput} - Input type for creating users
 *
 * ## Main Functions
 *
 * - {@link createUser} - Creates a new user
 * - {@link getUserById} - Retrieves a user by ID
 * - {@link updateUser} - Updates an existing user
 * - {@link deleteUser} - Deletes a user
 *
 * ## Example Usage
 *
 * ```typescript
 * import { createUser, getUserById, type User } from './user';
 *
 * // Create a user
 * const user = await createUser({
 *   name: 'John Doe',
 *   email: 'john@example.com',
 * });
 *
 * // Retrieve a user
 * const fetchedUser = await getUserById(user.id);
 * ```
 *
 * @module user
 */

// Export types and functions...
```

---

## Common Anti-patterns

### Avoid Type Assertion Abuse

```typescript
// BAD - Excessive use of type assertions
function processData(data: unknown): string {
  return (data as any).value as string;
}

// GOOD - Proper type guards
function processData(data: unknown): Result<string, TypeError> {
  if (!isObject(data) || !('value' in data)) {
    return err(new TypeError('Invalid data structure'));
  }
  
  if (typeof data.value !== 'string') {
    return err(new TypeError('Value must be a string'));
  }
  
  return ok(data.value);
}
```

### Avoid Loose Types

```typescript
// BAD - Too generic
function processItems(items: unknown[]): unknown[] { /* ... */ }

// GOOD - Proper generics
function processItems<T>(items: readonly T[]): T[] { /* ... */ }

// BAD - Using any for callbacks
function executeCallback(callback: (data: any) => void): void { /* ... */ }

// GOOD - Proper typing
function executeCallback<T>(callback: (data: T) => void, data: T): void { /* ... */ }
```

### Avoid Optional Chaining Overuse

```typescript
// BAD - Unnecessary optional chaining
function getName(user: User): string {
  return user?.name ?? 'Unknown';
}

// GOOD - Direct property access
function getName(user: User): string {
  return user.name;
}

// BAD - Optional chaining when value is guaranteed
function getUserEmail(user?: User): string | undefined {
  return user?.email; // Redundant if user is optional
}

// GOOD - Type guard or default value
function getUserEmail(user: User | undefined): string {
  if (!user) return 'No email';
  return user.email;
}
```

### Avoid Loose Null Checks

```typescript
// BAD - Loose equality
if (value == null) { /* ... */ }

// GOOD - Strict equality
if (value === null || value === undefined) { /* ... */ }

// BAD - Relying on truthiness
if (data) { /* ... */ }

// GOOD - Explicit check
if (data !== null && data !== undefined) { /* ... */ }
```

---

## Code Review Checklist

### Type Safety

- [ ] No use of `any` (use `unknown` if truly unknown)
- [ ] No use of `@ts-ignore` or `@ts-expect-error`
- [ ] Minimal use of type assertions (`as`)
- [ ] Proper use of type guards for narrowing
- [ ] All functions have explicit return types (especially async)
- [ ] Generics have proper constraints
- [ ] Discriminated unions used for state machines

### Code Quality

- [ ] Functions are small and focused (< 50 lines)
- [ ] Functions have single responsibility
- [ ] Proper error handling with Result type or custom errors
- [ ] Proper use of `readonly` for immutability
- [ ] Consistent naming conventions
- [ ] Descriptive function and parameter names
- [ ] No code duplication

### Performance

- [ ] Proper memoization for expensive operations
- [ ] React components use `React.memo` when needed
- [ ] Proper use of `useCallback` and `useMemo`
- [ ] No unnecessary re-renders
- [ ] Efficient data structures
- [ ] Proper use of lazy loading

### Documentation

- [ ] All public APIs have TSDoc comments
- [ ] Complex types have explanation comments
- [ ] Examples provided for public functions
- [ ] Parameter and return types documented
- [ ] Error cases documented

### Testing

- [ ] Unit tests for all public functions
- [ ] Tests cover happy path and error cases
- [ ] Edge cases tested
- [ ] Type safety verified in tests
- [ ] Mocking is minimal and controlled

---

## Migration Strategies

### Incremental Adoption

```typescript
// Step 1: Enable strict mode gradually
{
  "compilerOptions": {
    "strictNullChecks": true,
    "noImplicitAny": true,
    // Add other strict options incrementally
  }
}

// Step 2: Replace any with unknown
// Before
function process(data: any): any { /* ... */ }

// After
function process(data: unknown): unknown { /* ... */ }

// Step 3: Add type guards
function processData(data: unknown): Result<ProcessedData, Error> {
  if (!isValidInput(data)) {
    return err(new Error('Invalid input'));
  }
  
  // Now we know data is valid
  return ok(transformData(data));
}
```

### Legacy Code Integration

```typescript
// Use declaration merging for existing JS
declare global {
  interface Window {
    readonly analytics?: {
      readonly track: (event: string, data: unknown) => void;
    };
  }
}

// Create wrapper for untyped APIs
class AnalyticsWrapper {
  track(event: string, data: Record<string, unknown>): void {
    if (window.analytics) {
      window.analytics.track(event, data);
    }
  }
}
```

---

## Conclusion

Following these best practices ensures your TypeScript code maintains FAANG-level quality:

1. **Type Safety First** - Leverage the type system to catch errors at compile time
2. **Write Self-Documenting Code** - Types serve as documentation
3. **Eliminate Impossible States** - Use types to prevent bugs
4. **Refactor with Confidence** - Strong types enable fearless refactoring
5. **Focus on Developer Experience** - Types should make code easier to read and write

Remember: TypeScript is a tool that, when used correctly, significantly improves code quality and developer productivity. Invest in learning its advanced features and apply them consistently across your codebase.

For questions or clarifications, reach out to the Engineering Standards team.