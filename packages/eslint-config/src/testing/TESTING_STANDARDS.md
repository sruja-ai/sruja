# Sruja Testing Standards - FAANG Quality Guide

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Coverage Requirements](#test-coverage-requirements)
3. [Test Organization](#test-organization)
4. [Unit Testing Standards](#unit-testing-standards)
5. [React Component Testing](#react-component-testing)
6. [Integration Testing](#integration-testing)
7. [End-to-End Testing](#end-to-end-testing)
8. [Performance Testing](#performance-testing)
9. [Accessibility Testing](#accessibility-testing)
10. [Mocking and Test Doubles](#mocking-and-test-doubles)
11. [Test Data Management](#test-data-management)
12. [CI/CD Integration](#cicd-integration)
13. [Test Naming Conventions](#test-naming-conventions)
14. [Common Pitfalls](#common-pitfalls)

---

## Testing Philosophy

### Core Principles

At Sruja, we follow the **Testing Pyramid** philosophy, but with a FAANG-level emphasis on quality and reliability:

```
         ┌─────────────┐
         │   E2E       │  10% - Critical user flows
         │   Tests     │
         ├─────────────┤
         │ Integration │  30% - Component interaction
         │   Tests     │
         ├─────────────┤
         │    Unit     │  60% - Isolated business logic
         │   Tests     │
         └─────────────┘
```

### Quality Standards

- **Reliability**: Tests must be deterministic - same inputs always produce same outputs
- **Speed**: Unit tests should run in < 100ms, integration tests in < 1s
- **Maintainability**: Tests should be easy to understand and modify
- **Isolation**: Tests should not depend on external state or each other
- **Clarity**: Test names and assertions should be self-documenting
- **Comprehensive**: Cover happy paths, error paths, and edge cases

### Test-Driven Development (TDD)

We **strongly encourage** TDD for new features:

1. **Red**: Write a failing test
2. **Green**: Write minimal code to pass the test
3. **Refactor**: Improve the code while keeping tests green

**When TDD is required:**
- Public APIs and libraries
- Complex business logic
- Security-critical code paths
- Data transformations

---

## Test Coverage Requirements

### Coverage Targets

| Layer | Type | Minimum Coverage | Recommended |
|-------|------|------------------|-------------|
| **Packages** | Shared libraries | 90% | 95% |
| **UI Components** | React components | 80% | 90% |
| **Business Logic** | Services, utils | 95% | 100% |
| **Critical Paths** | Authentication, payments | 100% | 100% |
| **Overall** | Project-wide | 80% | 85% |

### Coverage Metrics

We track these metrics in CI:

```typescript
// coverage thresholds
{
  "statements": 80,
  "branches": 80,
  "functions": 80,
  "lines": 80
}
```

### Critical Code Coverage

**100% coverage required for:**
- Authentication and authorization
- Payment processing
- Data encryption/decryption
- Input validation
- Error handling in critical paths
- Security utilities

**95% coverage required for:**
- Public API surface
- Type converters
- Data transformations
- State management
- Routing logic

---

## Test Organization

### File Structure

```
src/
├── components/
│   ├── Button/
│   │   ├── Button.tsx
│   │   ├── Button.test.tsx
│   │   └── index.ts
│   └── Form/
│       ├── Form.tsx
│       ├── Form.test.tsx
│       └── __tests__/
│           ├── Form.integration.test.tsx
│           └── Form.a11y.test.tsx
├── services/
│   ├── api.ts
│   ├── api.test.ts
│   └── __mocks__/
│       └── api.mock.ts
└── utils/
    ├── date.ts
    ├── date.test.ts
    └── date.test.utils.ts
```

### Test File Naming

**Unit Tests:**
- Same name as source file with `.test` suffix
- Example: `Button.tsx` → `Button.test.tsx`

**Integration Tests:**
- Use `.integration.test` suffix
- Place in `__tests__` directory
- Example: `__tests__/Checkout.integration.test.tsx`

**E2E Tests:**
- Use `.e2e.test` suffix
- Place in `tests/` directory at project root
- Example: `tests/checkout.e2e.test.ts`

### Test Structure

Each test file should follow this structure:

```typescript
// 1. Imports (grouped and sorted)
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from './Button';

// 2. Types (if needed)
type ButtonProps = React.ComponentProps<typeof Button>;

// 3. Constants and test data
const DEFAULT_PROPS: ButtonProps = {
  children: 'Click me',
  onClick: vi.fn(),
};

// 4. Helper functions
const renderButton = (props: Partial<ButtonProps> = {}) => {
  return render(<Button {...DEFAULT_PROPS} {...props} />);
};

// 5. Test suite
describe('Button', () => {
  // 6. Setup/teardown
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // 7. Test groups
  describe('Rendering', () => {
    it('renders children correctly', () => {
      renderButton({ children: 'Save' });
      expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
    });
  });

  // 8. Teardown
  afterEach(() => {
    vi.restoreAllMocks();
  });
});
```

---

## Unit Testing Standards

### AAA Pattern (Arrange-Act-Assert)

All unit tests should follow the AAA pattern:

```typescript
it('calculates total price with tax', () => {
  // Arrange - Set up test data and conditions
  const price = 100;
  const taxRate = 0.08;
  const expectedTotal = 108;

  // Act - Execute the function being tested
  const result = calculateTotal(price, taxRate);

  // Assert - Verify the result
  expect(result).toBe(expectedTotal);
});
```

### Test Organization

**Given-When-Then for BDD-style tests:**

```typescript
describe('calculateTotal', () => {
  describe('when given valid inputs', () => {
    it('should return the total including tax', () => {
      // Given
      const price = 100;
      const taxRate = 0.08;

      // When
      const result = calculateTotal(price, taxRate);

      // Then
      expect(result).toBe(108);
    });
  });

  describe('when given zero price', () => {
    it('should return zero', () => {
      const result = calculateTotal(0, 0.08);
      expect(result).toBe(0);
    });
  });
});
```

### Testing Pure Functions

Pure functions should have exhaustive tests:

```typescript
describe('formatCurrency', () => {
  const testCases = [
    { input: 100, currency: 'USD', expected: '$100.00' },
    { input: 100.5, currency: 'USD', expected: '$100.50' },
    { input: 100.123, currency: 'USD', expected: '$100.12' },
    { input: -50, currency: 'USD', expected: '-$50.00' },
    { input: 100, currency: 'EUR', expected: '€100.00' },
  ];

  testCases.forEach(({ input, currency, expected }) => {
    it(`should format ${input} ${currency} as ${expected}`, () => {
      expect(formatCurrency(input, currency)).toBe(expected);
    });
  });
});
```

### Testing Async Code

**Always handle async properly:**

```typescript
// BAD - Missing await
it('fetches user data', async () => {
  fetchUser(1).then(user => {
    expect(user.name).toBe('John');
  });
});

// GOOD - Using async/await
it('fetches user data', async () => {
  const user = await fetchUser(1);
  expect(user.name).toBe('John');
});

// GOOD - Using Promise
it('fetches user data', () => {
  return expect(fetchUser(1)).resolves.toHaveProperty('name', 'John');
});
```

### Testing Error Handling

Always test error paths:

```typescript
describe('validateEmail', () => {
  it('should accept valid emails', () => {
    expect(() => validateEmail('user@example.com')).not.toThrow();
  });

  it('should reject invalid emails', () => {
    expect(() => validateEmail('invalid')).toThrow(ValidationError);
    expect(() => validateEmail('invalid')).toThrow('Invalid email format');
  });

  it('should reject empty strings', () => {
    expect(() => validateEmail('')).toThrow(ValidationError);
  });

  it('should reject null values', () => {
    expect(() => validateEmail(null as any)).toThrow(TypeError);
  });
});
```

### Testing Generics

Test with various type parameters:

```typescript
describe('ArrayUtils.unique', () => {
  it('should deduplicate numbers', () => {
    expect(unique([1, 2, 2, 3])).toEqual([1, 2, 3]);
  });

  it('should deduplicate strings', () => {
    expect(unique(['a', 'b', 'b', 'c'])).toEqual(['a', 'b', 'c']);
  });

  it('should deduplicate objects by reference', () => {
    const obj1 = { id: 1 };
    const obj2 = { id: 1 };
    expect(unique([obj1, obj2])).toHaveLength(2); // Different references
    expect(unique([obj1, obj1])).toHaveLength(1); // Same reference
  });
});
```

---

## React Component Testing

### Testing Component Rendering

```typescript
import { render, screen } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
  it('renders children text', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole('button', { name: 'Click me' })).toBeInTheDocument();
  });

  it('renders with custom class name', () => {
    render(<Button className="custom-class">Click me</Button>);
    expect(screen.getByRole('button')).toHaveClass('custom-class');
  });

  it('renders disabled state', () => {
    render(<Button disabled>Click me</Button>);
    expect(screen.getByRole('button')).toBeDisabled();
  });
});
```

### Testing User Interactions

```typescript
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Counter } from './Counter';

describe('Counter', () => {
  it('increments count when increment button is clicked', async () => {
    const user = userEvent.setup();
    render(<Counter />);

    const incrementButton = screen.getByRole('button', { name: /increment/i });
    const countDisplay = screen.getByTestId('count');

    expect(countDisplay).toHaveTextContent('0');

    await user.click(incrementButton);
    expect(countDisplay).toHaveTextContent('1');

    await user.click(incrementButton);
    expect(countDisplay).toHaveTextContent('2');
  });

  it('decrements count when decrement button is clicked', async () => {
    const user = userEvent.setup();
    render(<Counter initialValue={5} />);

    const decrementButton = screen.getByRole('button', { name: /decrement/i });
    const countDisplay = screen.getByTestId('count');

    await user.click(decrementButton);
    expect(countDisplay).toHaveTextContent('4');
  });
});
```

### Testing Props

```typescript
describe('Button', () => {
  describe('variants', () => {
    it('applies primary variant styles', () => {
      render(<Button variant="primary">Click me</Button>);
      expect(screen.getByRole('button')).toHaveClass('btn-primary');
    });

    it('applies secondary variant styles', () => {
      render(<Button variant="secondary">Click me</Button>);
      expect(screen.getByRole('button')).toHaveClass('btn-secondary');
    });
  });

  describe('sizes', () => {
    it('applies small size styles', () => {
      render(<Button size="small">Click me</Button>);
      expect(screen.getByRole('button')).toHaveClass('btn-sm');
    });

    it('applies large size styles', () => {
      render(<Button size="large">Click me</Button>);
      expect(screen.getByRole('button')).toHaveClass('btn-lg');
    });
  });
});
```

### Testing Event Handlers

```typescript
describe('Button', () => {
  it('calls onClick handler when clicked', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={handleClick}>Click me</Button>);
    
    await user.click(screen.getByRole('button'));
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('does not call onClick when disabled', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={handleClick} disabled>Click me</Button>);
    
    await user.click(screen.getByRole('button'));
    
    expect(handleClick).not.toHaveBeenCalled();
  });
});
```

### Testing Forms

```typescript
describe('LoginForm', () => {
  it('submits form with valid data', async () => {
    const handleSubmit = vi.fn();
    const user = userEvent.setup();
    
    render(<LoginForm onSubmit={handleSubmit} />);
    
    // Fill in form fields
    await user.type(screen.getByLabelText(/email/i), 'user@example.com');
    await user.type(screen.getByLabelText(/password/i), 'password123');
    
    // Submit form
    await user.click(screen.getByRole('button', { name: /sign in/i }));
    
    // Verify submission
    await waitFor(() => {
      expect(handleSubmit).toHaveBeenCalledWith({
        email: 'user@example.com',
        password: 'password123',
      });
    });
  });

  it('shows validation errors for invalid data', async () => {
    const user = userEvent.setup();
    
    render(<LoginForm onSubmit={vi.fn()} />);
    
    // Try to submit without filling fields
    await user.click(screen.getByRole('button', { name: /sign in/i }));
    
    // Check for error messages
    expect(screen.getByText(/email is required/i)).toBeInTheDocument();
    expect(screen.getByText(/password is required/i)).toBeInTheDocument();
  });
});
```

### Testing Async Components

```typescript
describe('UserProfile', () => {
  it('shows loading state initially', () => {
    render(<UserProfile userId={1} />);
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('displays user data after loading', async () => {
    render(<UserProfile userId={1} />);
    
    await waitFor(() => {
      expect(screen.getByText('John Doe')).toBeInTheDocument();
    });
    
    expect(screen.queryByTestId('loading-spinner')).not.toBeInTheDocument();
  });

  it('shows error message on fetch failure', async () => {
    // Mock failed fetch
    vi.mocked(fetchUser).mockRejectedValue(new Error('Network error'));
    
    render(<UserProfile userId={999} />);
    
    await waitFor(() => {
      expect(screen.getByText(/failed to load user/i)).toBeInTheDocument();
    });
  });
});
```

### Testing Custom Hooks

```typescript
import { renderHook, act, waitFor } from '@testing-library/react';
import { useCounter } from './useCounter';

describe('useCounter', () => {
  it('should initialize with default value', () => {
    const { result } = renderHook(() => useCounter());
    expect(result.current.count).toBe(0);
  });

  it('should increment count', () => {
    const { result } = renderHook(() => useCounter());
    
    act(() => {
      result.current.increment();
    });
    
    expect(result.current.count).toBe(1);
  });

  it('should handle async increment', async () => {
    const { result } = renderHook(() => useCounter());
    
    await act(async () => {
      await result.current.asyncIncrement();
    });
    
    await waitFor(() => {
      expect(result.current.count).toBe(1);
    });
  });
});
```

### Testing Context Providers

```typescript
describe('AuthContext', () => {
  it('provides authentication state', () => {
    const wrapper = ({ children }) => (
      <AuthProvider>{children}</AuthProvider>
    );
    
    const { result } = renderHook(() => useAuth(), { wrapper });
    
    expect(result.current.isAuthenticated).toBe(false);
  });

  it('updates authentication state on login', async () => {
    const wrapper = ({ children }) => (
      <AuthProvider>{children}</AuthProvider>
    );
    
    const { result } = renderHook(() => useAuth(), { wrapper });
    
    await act(async () => {
      await result.current.login('user@example.com', 'password');
    });
    
    expect(result.current.isAuthenticated).toBe(true);
  });
});
```

### Testing React Query Queries

```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const createTestQueryClient = () => {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
};

describe('useUserProfile', () => {
  it('fetches user profile data', async () => {
    const queryClient = createTestQueryClient();
    const wrapper = ({ children }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    
    const { result } = renderHook(() => useUserProfile(1), { wrapper });
    
    expect(result.current.isLoading).toBe(true);
    
    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    
    expect(result.current.data).toEqual({
      id: 1,
      name: 'John Doe',
      email: 'john@example.com',
    });
  });
});
```

### Accessibility Testing

```typescript
describe('Button Accessibility', () => {
  it('has proper ARIA attributes', () => {
    render(<Button aria-label="Close dialog" onClick={vi.fn()} />);
    expect(screen.getByLabelText('Close dialog')).toBeInTheDocument();
  });

  it('has proper keyboard support', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={handleClick}>Click me</Button>);
    
    const button = screen.getByRole('button');
    button.focus();
    expect(button).toHaveFocus();
    
    await user.keyboard('{Enter}');
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

---

## Integration Testing

### Component Integration Tests

```typescript
describe('ShoppingCart Integration', () => {
  it('adds product to cart and updates total', async () => {
    const user = userEvent.setup();
    
    render(
      <CartProvider>
        <ProductList />
        <CartSummary />
      </CartProvider>
    );
    
    // Add product to cart
    await user.click(screen.getByRole('button', { name: /add to cart/i }));
    
    // Verify cart updated
    await waitFor(() => {
      expect(screen.getByTestId('cart-count')).toHaveTextContent('1');
    });
    
    // Verify total updated
    expect(screen.getByTestId('cart-total')).toHaveTextContent('$99.99');
  });
});
```

### API Integration Tests

```typescript
describe('User API Integration', () => {
  it('creates and fetches user', async () => {
    const testUser = {
      email: 'test@example.com',
      name: 'Test User',
    };
    
    // Create user
    const createdUser = await api.users.create(testUser);
    expect(createdUser.id).toBeDefined();
    expect(createdUser.email).toBe(testUser.email);
    
    // Fetch user
    const fetchedUser = await api.users.get(createdUser.id);
    expect(fetchedUser).toEqual(createdUser);
    
    // Cleanup
    await api.users.delete(createdUser.id);
  });
});
```

---

## End-to-End Testing

### Playwright E2E Tests

```typescript
import { test, expect } from '@playwright/test';

test.describe('Checkout Flow', () => {
  test('complete checkout process', async ({ page }) => {
    // Navigate to product page
    await page.goto('/products/1');
    
    // Add to cart
    await page.click('button:has-text("Add to Cart")');
    await expect(page.locator('[data-testid="cart-count"]')).toHaveText('1');
    
    // Navigate to cart
    await page.click('[data-testid="cart-icon"]');
    await expect(page).toHaveURL('/cart');
    
    // Proceed to checkout
    await page.click('button:has-text("Checkout")');
    await expect(page).toHaveURL('/checkout');
    
    // Fill in shipping information
    await page.fill('[name="email"]', 'test@example.com');
    await page.fill('[name="address"]', '123 Test St');
    await page.fill('[name="city"]', 'Test City');
    await page.fill('[name="zip"]', '12345');
    
    // Place order
    await page.click('button:has-text("Place Order")');
    
    // Verify success
    await expect(page.locator('text=Order confirmed')).toBeVisible();
    await expect(page).toHaveURL('/order/success');
  });
});
```

---

## Performance Testing

### Component Performance

```typescript
describe('Component Performance', () => {
  it('should not re-render unnecessarily', () => {
    const renderSpy = vi.fn();
    const ExpensiveComponent = ({ value }: { value: number }) => {
      renderSpy();
      return <div>{value}</div>;
    };
    
    const { rerender } = render(<ExpensiveComponent value={1} />);
    expect(renderSpy).toHaveBeenCalledTimes(1);
    
    // Same prop value - should not re-render
    rerender(<ExpensiveComponent value={1} />);
    expect(renderSpy).toHaveBeenCalledTimes(1);
    
    // Different prop value - should re-render
    rerender(<ExpensiveComponent value={2} />);
    expect(renderSpy).toHaveBeenCalledTimes(2);
  });
});
```

---

## Accessibility Testing

### Axe-Core Integration

```typescript
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

describe('Accessibility', () => {
  it('should have no accessibility violations', async () => {
    const { container } = render(<App />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
```

---

## Mocking and Test Doubles

### Best Practices

```typescript
// GOOD - Mock only what you need
vi.mock('./api', () => ({
  fetchUser: vi.fn(),
}));

// BAD - Mock entire module
vi.mock('./api');

// GOOD - Use specific return values
vi.mocked(fetchUser).mockResolvedValue({ id: 1, name: 'John' });

// BAD - Use undefined returns
vi.mocked(fetchUser).mockResolvedValue(undefined as any);
```

---

## Test Data Management

### Test Factories

```typescript
// factories/user.ts
export const userFactory = (overrides = {}) => ({
  id: 1,
  email: 'user@example.com',
  name: 'John Doe',
  ...overrides,
});

// Usage
it('handles user data', () => {
  const user = userFactory({ email: 'custom@example.com' });
  expect(user.email).toBe('custom@example.com');
});
```

---

## CI/CD Integration

### Test Scripts

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "lint:test": "eslint '**/*.test.{ts,tsx}'"
  }
}
```

---

## Test Naming Conventions

### Descriptive Test Names

```typescript
// BAD
it('works', () => {});

// GOOD
it('renders user profile with correct name', () => {});

// EXCELLENT (Given-When-Then)
it('given a valid user ID, when the component mounts, then it displays the user profile', () => {});
```

---

## Common Pitfalls

### Avoid These

```typescript
// BAD - Testing implementation details
it('calls setState with value', () => {
  // Don't test internal methods
});

// GOOD - Testing behavior
it('updates display when button is clicked', () => {
  // Test what user sees
});

// BAD - Relying on sleep
await sleep(1000);

// GOOD - Using waitFor
await waitFor(() => {
  expect(element).toBeVisible();
});
```

---

## Conclusion

Following these standards ensures our codebase maintains FAANG-level quality. Remember:

1. **Test behavior, not implementation**
2. **Keep tests fast and reliable**
3. **Maintain high coverage of critical paths**
4. **Update tests when requirements change**
5. **Refactor tests along with code**

For questions or clarifications, reach out to the Engineering Standards team.