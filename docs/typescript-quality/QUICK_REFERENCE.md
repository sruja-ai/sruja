# TypeScript Quick Reference Guide
## Common Tasks and Patterns

## Table of Contents

- [Type Operations](#type-operations)
- [Type Guards](#type-guards)
- [Result Type](#result-type)
- [React Patterns](#react-patterns)
- [Generics](#generics)
- [Utility Types](#utility-types)
- [Async Patterns](#async-patterns)
- [Common Patterns](#common-patterns)
- [ESLint Quick Fixes](#eslint-quick-fixes)

---

## Type Operations

### Create Branded Types
```typescript
type UserId = string & { readonly __brand: unique symbol };
type Email = string & { readonly __brand: unique symbol };

// Factory function
const UserId = (id: string): UserId => id as UserId;
const Email = (email: string): Email => {
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    throw new Error('Invalid email');
  }
  return email as Email;
};

// Usage
const id = UserId('user-123');
const email = Email('user@example.com');
```

### Discriminated Unions
```typescript
type RequestState<T> =
  | { readonly status: 'idle' }
  | { readonly status: 'loading' }
  | { readonly status: 'success'; readonly data: T }
  | { readonly status: 'error'; readonly error: Error };

function render<T>(state: RequestState<T>) {
  switch (state.status) {
    case 'idle':
      return <Idle />;
    case 'loading':
      return <Loading />;
    case 'success':
      return <Success data={state.data} />; // TypeScript knows data exists
    case 'error':
      return <Error error={state.error} />; // TypeScript knows error exists
  }
}
```

### Template Literal Types
```typescript
type Color = 'red' | 'green' | 'blue';
type Size = 'sm' | 'md' | 'lg';
type ButtonClass = `btn-${Color}-${Size}`;

const button: ButtonClass = 'btn-red-md'; // ✅
// const invalid: ButtonClass = 'btn-xl'; // ❌
```

---

## Type Guards

### Primitive Type Guards
```typescript
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !isNaN(value);
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}
```

### Complex Type Guards
```typescript
interface User {
  readonly id: string;
  readonly name: string;
  readonly email: string;
}

function isUser(value: unknown): value is User {
  return (
    isObject(value) &&
    'id' in value &&
    isString(value.id) &&
    'name' in value &&
    isString(value.name) &&
    'email' in value &&
    isString(value.email)
  );
}

// Usage
function processUser(data: unknown) {
  if (isUser(data)) {
    console.log(data.name); // TypeScript knows data is User
  }
}
```

### Discriminated Union Type Guards
```typescript
type ApiResponse =
  | { readonly success: true; readonly data: User }
  | { readonly success: false; readonly error: Error };

function isSuccess(response: ApiResponse): response is { readonly success: true; readonly data: User } {
  return response.success;
}

function handleResponse(response: ApiResponse) {
  if (isSuccess(response)) {
    console.log(response.data.name); // TypeScript knows data exists
  } else {
    console.log(response.error.message); // TypeScript knows error exists
  }
}
```

---

## Result Type

### Basic Usage
```typescript
type Result<T, E = Error> = Ok<T> | Err<E>;

interface Ok<T> {
  readonly _tag: 'ok';
  readonly value: T;
}

interface Err<E> {
  readonly _tag: 'err';
  readonly error: E;
}

const ok = <T>(value: T): Ok<T> => ({ _tag: 'ok', value });
const err = <E>(error: E): Err<E> => ({ _tag: 'err', error });

const isOk = <T, E>(result: Result<T, E>): result is Ok<T> =>
  result._tag === 'ok';

const isErr = <T, E>(result: Result<T, E>): result is Err<E> =>
  result._tag === 'err';

// Usage
async function fetchUser(id: string): Promise<Result<User, ApiError>> {
  try {
    const response = await fetch(`/api/users/${id}`);
    
    if (!response.ok) {
      return err({ code: 'NOT_FOUND', message: 'User not found' });
    }
    
    const data = await response.json();
    return ok(data);
  } catch (error) {
    return err({ code: 'NETWORK_ERROR', message: 'Network error' });
  }
}

// Handling
const result = await fetchUser('123');

if (isOk(result)) {
  console.log(result.value.name); // Safe access
} else {
  console.error(result.error.message); // Safe access
}
```

### Chaining Results
```typescript
const map = <T, U, E>(result: Result<T, E>, fn: (value: T) => U): Result<U, E> =>
  isOk(result) ? ok(fn(result.value)) : result;

const andThen = <T, U, E>(
  result: Result<T, E>,
  fn: (value: T) => Result<U, E>
): Result<U, E> => isOk(result) ? fn(result.value) : result;

// Usage
async function getUserEmail(id: string): Promise<Result<string, ApiError>> {
  return fetchUser(id)
    .then(result => map(result, user => user.email))
    .then(result => andThen(result, validateEmail));
}
```

---

## React Patterns

### Component Props
```typescript
interface ButtonProps {
  readonly variant: 'primary' | 'secondary' | 'danger';
  readonly size: 'small' | 'medium' | 'large';
  readonly disabled?: boolean;
  readonly loading?: boolean;
  readonly onClick: () => void;
  readonly children: ReactNode;
}

export const Button: FC<ButtonProps> = ({
  variant,
  size,
  disabled = false,
  loading = false,
  onClick,
  children,
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
```

### Generic Component
```typescript
interface ListProps<T> {
  readonly items: readonly T[];
  readonly renderItem: (item: T) => ReactNode;
  readonly keyExtractor: (item: T) => string | number;
  readonly emptyMessage?: string;
}

export function List<T>({ items, renderItem, keyExtractor, emptyMessage }: ListProps<T>) {
  if (items.length === 0) {
    return <div>{emptyMessage || 'No items'}</div>;
  }

  return (
    <ul>
      {items.map(item => (
        <li key={keyExtractor(item)}>{renderItem(item)}</li>
      ))}
    </ul>
  );
}

// Usage
<List<User>
  items={users}
  renderItem={user => <div>{user.name}</div>}
  keyExtractor={user => user.id}
/>
```

### Custom Hook
```typescript
interface UseAsyncState<T> {
  readonly data: T | null;
  readonly error: Error | null;
  readonly isLoading: boolean;
}

function useAsync<T>(fn: () => Promise<T>): UseAsyncState<T> {
  const [state, setState] = useState<UseAsyncState<T>>({
    data: null,
    error: null,
    isLoading: true,
  });

  useEffect(() => {
    fn()
      .then(data => setState({ data, error: null, isLoading: false }))
      .catch(error => setState({ data: null, error, isLoading: false }));
  }, [fn]);

  return state;
}

// Usage
function UserProfile({ userId }: { readonly userId: string }) {
  const { data: user, isLoading, error } = useAsync(() => fetchUser(userId));

  if (isLoading) return <Spinner />;
  if (error) return <ErrorDisplay error={error} />;
  if (!user) return <Empty />;

  return <UserCard user={user} />;
}
```

---

## Generics

### Basic Generic Function
```typescript
function identity<T>(value: T): T {
  return value;
}

const str = identity('hello'); // string
const num = identity(42); // number
```

### Generic Constraints
```typescript
interface Lengthwise {
  readonly length: number;
}

function getLength<T extends Lengthwise>(arg: T): number {
  return arg.length;
}

getLength('hello'); // ✅
getLength([1, 2, 3]); // ✅
// getLength(42); // ❌
```

### Generic Class
```typescript
class Repository<T extends { readonly id: string }> {
  constructor(private readonly items: readonly T[]) {}

  findById(id: string): T | undefined {
    return this.items.find(item => item.id === id);
  }

  filter(predicate: (item: T) => boolean): readonly T[] {
    return this.items.filter(predicate);
  }
}

// Usage
const userRepo = new Repository<User>(users);
const user = userRepo.findById('123');
```

---

## Utility Types

### Common Utility Types
```typescript
// Make all properties optional
type PartialUser = Partial<User>;

// Make all properties required
type RequiredUser = Required<User>;

// Pick specific properties
type UserBasic = Pick<User, 'id' | 'name'>;

// Omit specific properties
type UserPrivate = Omit<User, 'password' | 'email'>;

// Make all properties readonly
type ReadonlyUser = Readonly<User>;

// Make properties nullable
type NullableUser = {
  [K in keyof User]: User[K] | null;
};
```

### Advanced Utility Types
```typescript
// Deep readonly
type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};

// Deep partial
type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

// Extract keys by type
type StringKeys<T> = {
  [K in keyof T]: T[K] extends string ? K : never;
}[keyof T];

// Make some properties optional
type OptionalSome<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

// Merge two types
type Merge<T, U> = Omit<T, keyof U> & U;
```

---

## Async Patterns

### Async/Await with Error Handling
```typescript
async function fetchWithTimeout<T>(
  fn: () => Promise<T>,
  timeoutMs: number
): Promise<Result<T, Error>> {
  try {
    const result = await Promise.race([
      fn(),
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error('Timeout')), timeoutMs)
      ),
    ]);
    return ok(result);
  } catch (error) {
    return err(error instanceof Error ? error : new Error(String(error)));
  }
}

// Usage
const result = await fetchWithTimeout(
  () => fetch('/api/data').then(r => r.json()),
  5000
);
```

### Retry Pattern
```typescript
async function retry<T>(
  fn: () => Promise<T>,
  options: {
    readonly maxAttempts?: number;
    readonly delay?: number;
    readonly shouldRetry?: (error: unknown) => boolean;
  } = {}
): Promise<T> {
  const {
    maxAttempts = 3,
    delay = 1000,
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

      await new Promise(resolve => setTimeout(resolve, delay * attempt));
    }
  }

  throw lastError;
}

// Usage
const data = await retry(() => fetchUser('123'), {
  maxAttempts: 5,
  delay: 1000,
  shouldRetry: (error) => !(error instanceof NotFoundError),
});
```

### Parallel Requests
```typescript
async function fetchMultipleData<T>(
  fetchers: readonly (() => Promise<T>)[]
): Promise<readonly Result<T, Error>[]> {
  const results = await Promise.allSettled(fetchers.map(fn => fn()));

  return results.map(result =>
    result.status === 'fulfilled'
      ? ok(result.value)
      : err(result.reason instanceof Error ? result.reason : new Error(String(result.reason)))
  );
}

// Usage
const results = await fetchMultipleData([
  () => fetchUser('1'),
  () => fetchUser('2'),
  () => fetchUser('3'),
]);

results.forEach((result, index) => {
  if (isOk(result)) {
    console.log(`User ${index + 1}:`, result.value);
  } else {
    console.error(`User ${index + 1} error:`, result.error);
  }
});
```

---

## Common Patterns

### Validate Input
```typescript
function validateConfig(config: unknown): Result<Config, ValidationError> {
  if (!isObject(config)) {
    return err({ field: 'config', message: 'Config must be an object' });
  }

  if (!('apiKey' in config) || !isString(config.apiKey)) {
    return err({ field: 'apiKey', message: 'Invalid API key' });
  }

  return ok({
    apiKey: config.apiKey,
    // ... other validated fields
  });
}

// Usage
const configResult = validateConfig(process.env.CONFIG);

if (isOk(configResult)) {
  const config = configResult.value;
  // Use validated config
} else {
  console.error('Configuration error:', configResult.error);
}
```

### Transform Data
```typescript
interface ApiUser {
  readonly id: string;
  readonly first_name: string;
  readonly last_name: string;
  readonly email_address: string;
}

interface User {
  readonly id: string;
  readonly name: string;
  readonly email: Email;
}

function transformApiUser(apiUser: ApiUser): User {
  return {
    id: apiUser.id,
    name: `${apiUser.first_name} ${apiUser.last_name}`,
    email: Email(apiUser.email_address),
  };
}

// Usage
const apiUser = await fetchUserFromApi();
const user = transformApiUser(apiUser);
```

### Memoize Function
```typescript
function memoize<T extends (...args: readonly unknown[]) => unknown>(
  fn: T,
  keyGenerator?: (...args: Parameters<T>) => string
): T {
  const cache = new Map<string, ReturnType<T>>();

  return ((...args: Parameters<T>) => {
    const key = keyGenerator
      ? keyGenerator(...args)
      : JSON.stringify(args);

    if (cache.has(key)) {
      return cache.get(key)!;
    }

    const result = fn(...args);
    cache.set(key, result);
    return result;
  }) as T;
}

// Usage
const expensiveCalculation = memoize((input: string) => {
  // Expensive computation
  return input.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
});
```

---

## ESLint Quick Fixes

### Fix `no-explicit-any`
```typescript
// ❌ BAD
function processData(data: any): string {
  return data.name;
}

// ✅ GOOD - Use proper type
interface Data {
  readonly name: string;
}

function processData(data: Data): string {
  return data.name;
}

// ✅ GOOD - Use unknown with type guard
function processData(data: unknown): Result<string, TypeError> {
  if (!isObject(data) || !('name' in data) || !isString(data.name)) {
    return err(new TypeError('Invalid data'));
  }
  return ok(data.name);
}
```

### Fix `no-magic-numbers`
```typescript
// ❌ BAD
function calculateArea(radius: number): number {
  return 3.14159 * radius * radius;
}

// ✅ GOOD - Extract constant
const PI = 3.14159;

function calculateArea(radius: number): number {
  return PI * radius * radius;
}

// ✅ GOOD - Use enum
const MathConstants = {
  PI: 3.14159,
  E: 2.71828,
} as const;

function calculateArea(radius: number): number {
  return MathConstants.PI * radius * radius;
}
```

### Fix `no-floating-promises`
```typescript
// ❌ BAD
async function fetchData(): Promise<void> {
  fetch('/api/data'); // Promise not handled
}

// ✅ GOOD - Handle promise
async function fetchData(): Promise<void> {
  await fetch('/api/data');
}

// ✅ GOOD - Use .then/catch
async function fetchData(): Promise<void> {
  fetch('/api/data')
    .then(response => console.log('Success'))
    .catch(error => console.error('Error:', error));
}

// ✅ GOOD - Explicitly ignore
async function fetchData(): Promise<void> {
  void fetch('/api/data'); // Explicitly ignore promise
}
```

### Fix `react-hooks/exhaustive-deps`
```typescript
// ❌ BAD
function Component({ userId }: { readonly userId: string }) {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    fetchUser(userId).then(setUser);
  }, []); // Missing userId dependency
}

// ✅ GOOD - Include dependency
function Component({ userId }: { readonly userId: string }) {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    fetchUser(userId).then(setUser);
  }, [userId]); // Correct dependency
}

// ✅ GOOD - Use useCallback for event handlers
function Component({ userId }: { readonly userId: string }) {
  const [user, setUser] = useState<User | null>(null);

  const handleClick = useCallback(() => {
    console.log('User:', user);
  }, [user]);

  useEffect(() => {
    fetchUser(userId).then(setUser);
  }, [userId]);

  return <button onClick={handleClick}>Click me</button>;
}
```

### Fix `jsx-no-literals`
```typescript
// ❌ BAD
function Button(): ReactNode {
  return <button>Click me</button>;
}

// ✅ GOOD - Use props
interface ButtonProps {
  readonly children: ReactNode;
}

function Button({ children }: ButtonProps): ReactNode {
  return <button>{children}</button>;
}

// ✅ GOOD - Use constant
const BUTTON_TEXT = 'Click me';

function Button(): ReactNode {
  return <button>{BUTTON_TEXT}</button>;
}

// ✅ GOOD - Use i18n
function Button(): ReactNode {
  return <button>{t('button.click')}</button>;
}
```

---

## Additional Resources

- [TypeScript Best Practices](./TYPESCRIPT_BEST_PRACTICES.md)
- [Testing Standards](./TESTING_STANDARDS.md)
- [Implementation Guide](./IMPLEMENTATION_GUIDE.md)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Type ESLint Rules](https://typescript-eslint.io/rules/)

---

## Quick Tips

### Keyboard Shortcuts (VS Code)
- `F2` - Rename symbol
- `F12` - Go to definition
- `Shift+F12` - Peek definition
- `Ctrl+Space` - Trigger suggestions
- `Ctrl+.` - Quick fix
- `Ctrl+Shift+E` - Show errors

### Common Commands
```bash
# Type check
npx tsc --noEmit

# Format code
npx prettier --write "src/**/*.{ts,tsx}"

# Lint code
npm run lint

# Lint with auto-fix
npm run lint:fix

# Run tests
npm test

# Run tests with coverage
npm run test:coverage
```

### Debugging Types
```typescript
// Use `as` sparingly and with justification
// const data = response as any; // ❌

// Use type guards instead
if (isValidData(response)) {
  // ✅ TypeScript knows response is valid
}

// Use typeof checks
if (typeof value === 'string') {
  // ✅ TypeScript knows value is string
}

// Use instance checks
if (error instanceof Error) {
  // ✅ TypeScript knows error is Error
}
```

---

**Remember**: This is a living document. Update it as you learn new patterns and best practices!