---
title: "Lesson 3: User Journeys"
weight: 3
summary: "Modeling behavioral scenarios and user experiences"
time: "5 min"
---

# Walking in Their Shoes: User Journeys

Ever watched someone use a product you built? You notice things you never would—confusing buttons, unclear error messages, workflows that don't make sense. Why? Because you built it from your perspective, not theirs.

User journeys (or scenarios) are your way to model the complete user experience—from their perspective. They show what users do, how your system responds, and what happens when things go right (and wrong).

In this lesson, you'll learn to create user journeys that capture the complete user experience. You'll discover how to model happy paths and error paths, document edge cases, and create scenarios that serve as both requirements and test cases.

Let's start by understanding what user journeys actually are.

## Learning Goals

By the end of this lesson, you'll be able to:

- Model user journeys and behavioral scenarios from user's perspective
- Use BDD-style scenarios to document requirements
- Model both happy paths and error paths
- Document edge cases and unusual scenarios
- Create scenarios that serve as test cases and documentation

## What Are User Journeys, Really?

User journeys (also called scenarios or behavioral flows) show how a person interacts with your system to achieve a goal. Unlike data flows that show data moving through a system, user journeys show **human behavior** and **system responses**.

Think of it like a story: "As a customer, I want to buy a product so I can use it." The journey shows every step—what the customer does, what the system does, and what the customer experiences.

**User journeys capture:**
- User actions (what they click, type, select)
- System responses (what happens, what they see)
- Success paths (when everything works)
- Error paths (when things go wrong)
- Decision points (where different things happen based on conditions)

**What makes them special:**
- User's perspective (not system's)
- Behavioral (not just data)
- Complete experience (start to finish)
- Both happy and sad paths

## Why User Journeys Matter

I've learned this the hard way. I once built a feature without modeling user journeys, and when we launched, users were completely confused.

They couldn't find the checkout button. When they did, error messages were cryptic. The "success" page didn't tell them what to do next. Support tickets flooded in. We had to rebuild the entire feature.

If I'd modeled user journeys upfront, we would have seen these issues immediately. The journey would have shown: "User clicks checkout → System shows error 'ERR_500' → User is confused."

**User journeys matter because:**

### 1. Requirements Clarity

User journeys turn vague requirements into concrete scenarios:

**Vague requirement:** "Users should be able to check out"

**User journey makes it concrete:**
```sruja
// partial
CheckoutJourney = scenario "Customer Checkout" {
  Customer -> WebApp "Clicks checkout button"
  WebApp -> API "Validates cart"
  API -> Database "Checks inventory"
  Database -> API "Inventory available"
  API -> PaymentGateway "Processes payment"
  PaymentGateway -> API "Payment successful"
  API -> Database "Saves order"
  API -> EmailService "Sends confirmation"
  WebApp -> Customer "Shows order confirmation page"
}
```

Now everyone knows exactly what "checkout" means—every step, every system involved, every user action.

### 2. Test Case Generation

User journeys become test cases automatically:

```sruja
// partial
HappyPathTest = scenario "Test: Successful Checkout" {
  // From the user journey
  Customer -> WebApp "Clicks checkout"
  WebApp -> API "Validates cart"
  // ... test each step
}

ErrorPathTest = scenario "Test: Checkout with Invalid Payment" {
  Customer -> WebApp "Clicks checkout"
  WebApp -> API "Validates cart"
  API -> PaymentGateway "Processes payment"
  PaymentGateway -> API "Payment declined: invalid card"
  API -> WebApp "Returns error: Invalid card"
  WebApp -> Customer "Shows error: Please check your card details"
}
```

I've worked on teams where we spent weeks writing test cases manually. With user journeys, we just turn scenarios into tests. Huge time saver.

### 3. User Experience Visibility

User journeys show the complete experience—not just what works, but how it feels:

```sruja
// partial
RegistrationJourney = scenario "User Registration" {
  Customer -> WebApp "Opens registration form"
  Customer -> WebApp "Enters name and email"
  Customer -> WebApp "Enters password"
  Customer -> WebApp "Clicks 'Create Account'"
  WebApp -> API "Submits registration"
  API -> Database "Creates user account"
  API -> EmailService "Sends welcome email"
  WebApp -> Customer "Shows 'Account created!' message"
  WebApp -> Customer "Redirects to dashboard"
}
```

See how this shows the full experience? Form entry → submission → success message → welcome email → dashboard. Anyone reading this understands exactly what users experience.

### 4. Edge Case Discovery

When you model user journeys, you naturally think about edge cases:

```sruja
// partial
RegistrationEdgeCases = scenario "Registration Edge Cases" {
  // Edge case 1: Duplicate email
  Customer -> WebApp "Registers with existing email"
  WebApp -> API "Submits registration"
  API -> Database "Checks email exists"
  Database -> API "Email already exists"
  API -> WebApp "Returns error: Email already registered"
  WebApp -> Customer "Shows error: Email already in use"

  // Edge case 2: Weak password
  Customer -> WebApp "Registers with weak password"
  WebApp -> API "Submits registration"
  API -> ValidationService "Validates password strength"
  ValidationService -> API "Password too weak"
  API -> WebApp "Returns error: Password must be at least 8 characters"
  WebApp -> Customer "Shows error: Password too weak"
}
```

I once launched a feature without modeling edge cases. Users started using weird inputs, emojis in names, passwords with special characters, and the system broke in creative ways. Model edge cases upfront.

## Creating User Journeys in Sruja

Sruja gives you two keywords for user journeys, and they're interchangeable:

### Using `scenario`

Use `scenario` for behavioral flows—it's the most common and descriptive choice:

```sruja
// partial
CheckoutScenario = scenario "Customer Checkout Experience" {
  Customer -> Shop.WebApp "Clicks checkout button"
  Shop.WebApp -> Shop.API "Validates shopping cart"
  Shop.API -> PaymentGateway "Processes payment"
  Shop.WebApp -> Customer "Shows order confirmation"
}
```

### Using `story`

Use `story` when you want to emphasize the narrative or story aspect:

```sruja
// partial
CheckoutStory = story "As a customer, I want to checkout so I can purchase my items" {
  Customer -> Shop.WebApp "Clicks checkout button"
  Shop.WebApp -> Shop.API "Validates shopping cart"
  Shop.API -> PaymentGateway "Processes payment"
  Shop.WebApp -> Customer "Shows order confirmation"}
}
```

The `story` keyword is great for BDD (Behavior-Driven Development) style where you write scenarios in "Given-When-Then" format.

## BDD (Behavior-Driven Development) Style

BDD is about writing requirements as behavior, not specifications. User journeys in Sruja map perfectly to BDD's "Given-When-Then" structure.

### The Given-When-Then Pattern

```sruja
// partial
// GIVEN: Customer has items in shopping cart
// WHEN: Customer clicks checkout and completes payment
// THEN: Order is created and confirmation is shown

CheckoutJourney = scenario "Customer Checkout (BDD Style)" {
  // GIVEN: Customer has items in cart
  Customer -> Shop.WebApp "Views shopping cart with 3 items"
  Shop.WebApp -> Shop.API "Fetches cart contents"
  Shop.API -> Shop.Database "Returns cart data"
  
  // WHEN: Customer completes checkout flow
  Customer -> Shop.WebApp "Clicks checkout button"
  Shop.WebApp -> Shop.API "Validates cart"
  Shop.API -> Shop.Database "Checks inventory"
  Shop.Database -> Shop.API "All items available"
  Shop.API -> PaymentGateway "Processes payment"
  PaymentGateway -> Shop.API "Payment successful"
  
  // THEN: Order is created and confirmation shown
  Shop.API -> Shop.Database "Creates order"
  Shop.API -> Shop.Database "Reserves inventory"
  Shop.API -> EmailService "Sends confirmation email"
  Shop.WebApp -> Customer "Shows order confirmation page with order #12345"
}
```

**Why BDD works:**
- It's in plain language anyone can understand
- It focuses on behavior, not implementation
- It serves as both requirements and tests
- It's unambiguous (unlike "system should work well")

I've worked with product managers who couldn't understand technical specs. But they understood BDD scenarios immediately. It became our common language.

## User Journey Patterns

After modeling user journeys for years, I've found patterns that repeat constantly. Let me show you the ones I see most often.

### Pattern 1: Happy Path

The ideal scenario where everything works perfectly:

```sruja
// partial
HappyRegistration = scenario "User Registration (Happy Path)" {
  Customer -> WebApp "Opens registration form"
  Customer -> WebApp "Enters name, email, password"
  Customer -> WebApp "Clicks 'Create Account'"
  WebApp -> API "Submits registration"
  API -> Database "Creates user account"
  Database -> API "User created successfully"
  API -> EmailService "Sends welcome email"
  EmailService -> Customer "Receives welcome email"
  API -> WebApp "Returns success"
  WebApp -> Customer "Shows 'Account created! Welcome!' message"
  WebApp -> Customer "Redirects to dashboard"
}
```

**Characteristics:**
- No errors
- No branches
- Perfect flow from start to finish
- User achieves their goal

**When to model:**
- First—model the ideal scenario
- Before optimizing, understand what success looks like
- Before testing edge cases, have a working baseline

### Pattern 2: Error Path

What happens when things go wrong:

```sruja
// partial
ErrorRegistration = scenario "User Registration (Error Path: Duplicate Email)" {
  Customer -> WebApp "Opens registration form"
  Customer -> WebApp "Enters existing email: john@example.com"
  Customer -> WebApp "Enters name and password"
  Customer -> WebApp "Clicks 'Create Account'"
  WebApp -> API "Submits registration"
  API -> Database "Checks if email exists"
  Database -> API "Email already exists: john@example.com"
  API -> WebApp "Returns error: Email already registered"
  WebApp -> Customer "Shows error: 'Email already in use. Try logging in or use a different email.'"
}
```

**Common error paths to model:**
- Duplicate data (email, username)
- Invalid data (email format, weak password)
- Missing data (required fields empty)
- System errors (database down, API timeout)
- Business rule violations (age restriction, region blocking)

**Characteristics:**
- Error occurs at some step
- System returns meaningful error
- User sees helpful error message
- User can correct and retry

**When to model:**
- Always model critical error paths
- Focus on errors users actually encounter
- Ensure error messages are helpful, not cryptic

I once worked on a system where error messages were like "ERR_500_CHECKOUT_FAILED." Users had no idea what went wrong. We rewrote them to be helpful: "Payment failed. Please check your card details or try a different payment method." Support tickets dropped by 70%.

### Pattern 3: Branching Path

Different things happen based on conditions:

```sruja
// partial
BranchingApproval = scenario "Order Approval (Branching Based on Value)" {
  Manager -> WebApp "Submits order for approval"
  WebApp -> API "Processes approval request"
  API -> Database "Fetches order details"
  Database -> API "Returns order: value = $1500"
  
  // Branch 1: Auto-approve for low-value orders
  if value < 1000 {
    API -> Database "Updates order: auto-approved"
    API -> WebApp "Returns: Order auto-approved"
    WebApp -> Manager "Shows 'Order approved! Shipping soon.'"
  }
  
  // Branch 2: Manual review for high-value orders
  if value >= 1000 {
    API -> EmailService "Sends approval request to director"
    EmailService -> Director "Receives approval request"
    Director -> WebApp "Reviews order and approves"
    WebApp -> API "Submits approval decision"
    API -> Database "Updates order: manually approved"
    API -> WebApp "Returns: Order approved by director"
    WebApp -> Manager "Shows 'Order approved by director. Shipping soon.'"
  }
}
```

**Characteristics:**
- Decision point in the flow
- Multiple possible paths
- Different actions based on conditions
- Paths may converge at the end

**When to model:**
- Business logic has rules
- Different user types have different experiences
- System behavior changes based on context

### Pattern 4: Retry Path

System tries multiple times before succeeding or failing:

```sruja
// partial
RetryPayment = scenario "Payment with Automatic Retry" {
  Customer -> WebApp "Clicks 'Place Order'"
  WebApp -> API "Submits order for payment"
  
  // Attempt 1: Fails with timeout
  API -> PaymentGateway "Process payment"
  PaymentGateway -> API "Payment failed: timeout"
  API -> WebApp "Returns: Processing, please wait..."
  WebApp -> Customer "Shows spinner: 'Processing your payment...'"
  
  // Attempt 2: Fails with timeout
  API -> PaymentGateway "Retry payment (attempt 2)"
  PaymentGateway -> API "Payment failed: timeout"
  
  // Attempt 3: Succeeds
  API -> PaymentGateway "Retry payment (attempt 3)"
  PaymentGateway -> API "Payment successful!"
  
  // Continue with success
  API -> Database "Saves order"
  API -> EmailService "Sends confirmation"
  WebApp -> Customer "Shows order confirmation"
}
```

**Characteristics:**
- Same action repeated
- Eventually succeeds or fails permanently
- User sees "retrying" state
- Transparent about what's happening

**When to model:**
- External services are flaky
- Network issues are common
- You want to handle transient failures gracefully

## Complete User Journey Example

Let me show you a complete user journey that brings everything together:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

// Person
Customer = person "Customer"

// System
Shop = system "Shop" {
  WebApp = container "Web Application"
  API = container "API Service"
  Database = database "PostgreSQL"
}

// External systems
PaymentGateway = system "Payment Gateway" {
  metadata {
    tags ["external", "vendor"]
    sla "99.9% uptime"
  }
}

EmailService = system "Email Service" {
  metadata {
    tags ["external", "vendor"]
  }
}

// Complete user journey from browsing to confirmation
CompleteCheckoutJourney = scenario "Complete Checkout Experience" {
  // Step 1: Browse products
  Customer -> Shop.WebApp "Browses products"
  Shop.WebApp -> Shop.API "Fetches product list"
  Shop.API -> Shop.Database "Queries products"
  Shop.Database -> Shop.API "Returns 50 products"
  Shop.API -> Shop.WebApp "Returns product data"
  Shop.WebApp -> Customer "Displays products grid"

  // Step 2: Add to cart
  Customer -> Shop.WebApp "Clicks 'Add to Cart' on Product #5"
  Shop.WebApp -> Shop.API "Adds item to cart"
  Shop.API -> Shop.Database "Saves cart item"

  // Step 3: Review cart
  Customer -> Shop.WebApp "Clicks 'View Cart'"
  Shop.WebApp -> Shop.API "Fetches cart contents"
  Shop.API -> Shop.Database "Returns cart: 3 items, $75 total"
  Shop.API -> Shop.WebApp "Returns cart data"
  Shop.WebApp -> Customer "Shows cart with total"

  // Step 4: Checkout
  Customer -> Shop.WebApp "Clicks 'Checkout'"
  Shop.WebApp -> Shop.API "Initiates checkout"
  Shop.API -> Shop.Database "Validates cart"
  Shop.Database -> Shop.API "Cart is valid"
  
  // Step 5: Payment processing
  Shop.API -> PaymentGateway "Process payment: $75"
  PaymentGateway -> Shop.API "Payment successful!"
  
  // Step 6: Order creation
  Shop.API -> Shop.Database "Creates order #12345"
  Shop.API -> Shop.Database "Reserves inventory"
  Shop.API -> EmailService "Send order confirmation"
  
  // Step 7: Confirmation page
  Shop.API -> Shop.WebApp "Returns order confirmation"
  Shop.WebApp -> Customer "Shows 'Order #12345 Confirmed!' page"
  
  // Step 8: Email delivery
  EmailService -> Customer "Sends confirmation email"
}

view index {
  include *
}
```

This journey shows the complete experience from start to finish—every user action, every system response, every step. Anyone reading this understands exactly what a customer experiences when checking out.

## Testing with Scenarios

One of the most powerful things about user journeys: they become test cases automatically.

### Acceptance Criteria as Scenarios

```sruja
// partial
// Acceptance criteria: As a customer, I want to checkout so that I can purchase products

// AC1: Customer can checkout with valid payment
HappyCheckout = scenario "AC1: Successful Checkout" {
  Customer -> Shop.WebApp "Clicks checkout with valid payment"
  Shop.WebApp -> Shop.API "Validates cart"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment successful"
  Shop.API -> Shop.Database "Creates order"
  Shop.API -> EmailService "Sends confirmation"
  Shop.WebApp -> Customer "Shows confirmation page"
}

// AC2: Customer sees helpful error with invalid payment
InvalidPaymentCheckout = scenario "AC2: Checkout with Invalid Payment" {
  Customer -> Shop.WebApp "Clicks checkout with expired card"
  Shop.WebApp -> Shop.API "Validates cart"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment declined: card expired"
  Shop.API -> Shop.WebApp "Returns error"
  Shop.WebApp -> Customer "Shows 'Card expired. Please update your payment method.'"
}

// AC3: Customer receives confirmation email
EmailConfirmation = scenario "AC3: Confirmation Email Sent" {
  Shop.API -> Shop.Database "Creates order"
  Shop.API -> EmailService "Send confirmation"
  EmailService -> Customer "Receives confirmation email"}
```

I've worked on teams where we spent months debating requirements. When we turned them into BDD-style scenarios, everyone agreed. No ambiguity, no confusion, no "I thought you meant X."

## Documenting Edge Cases

Don't just model happy paths. Edge cases are where systems break.

### Common Edge Cases to Model

```sruja
// partial
// Edge case 1: Checkout with expired card
ExpiredCardCheckout = scenario "Checkout with Expired Card" {
  Customer -> Shop.WebApp "Attempts checkout with expired card"
  Shop.WebApp -> Shop.API "Submits order"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Payment failed: card expired"
  Shop.API -> Shop.WebApp "Returns error"
  Shop.WebApp -> Customer "Shows 'Your card has expired. Please update your payment method.'"
}

// Edge case 2: Checkout with insufficient inventory
InsufficientInventoryCheckout = scenario "Checkout with Insufficient Inventory" {
  Customer -> Shop.WebApp "Attempts checkout"
  Shop.WebApp -> Shop.API "Submits order"
  Shop.API -> Shop.Database "Checks inventory"
  Shop.Database -> Shop.API "Insufficient stock: 5 requested, 2 available"
  Shop.API -> Shop.WebApp "Returns error"
  Shop.WebApp -> Customer "Shows 'Sorry, only 2 items available. Would you like to proceed with 2?'"
}

// Edge case 3: Checkout during payment gateway outage
GatewayOutageCheckout = scenario "Checkout During Payment Outage" {
  Customer -> Shop.WebApp "Attempts checkout"
  Shop.WebApp -> Shop.API "Submits order"
  Shop.API -> PaymentGateway "Process payment"
  PaymentGateway -> Shop.API "Service unavailable: outage in progress"
  Shop.API -> Shop.WebApp "Returns error"
  Shop.WebApp -> Customer "Shows 'Payment service temporarily unavailable. Please try again in a few minutes. We've saved your cart.'"
}
```

**Why model edge cases:**
- They expose gaps in your design
- They become test cases automatically
- They help teams discuss "what if" scenarios
- They prevent surprises in production

I once launched a feature without modeling edge cases. Users immediately found scenarios we hadn't considered—checking out with gift cards during sales, checking out from different countries with different currencies, checking out with addresses that don't validate. We spent months fixing edge cases we could have caught upfront.

## What to Remember

User journeys tell the story of how users interact with your system—from their perspective. When you create user journeys:

- **Focus on user's experience** — Not just what works, but how it feels
- **Model both paths** — Happy paths AND error paths
- **Use BDD style** — "Given-When-Then" for clarity
- **Document edge cases** — Unusual but important scenarios
- **Make them testable** — Each scenario becomes a test case
- **Write helpful errors** — Users should understand what went wrong

If you take away one thing, let it be this: **user journeys are your best tool for ensuring your system actually works for real humans, not just in theory**. They bridge the gap between requirements, testing, and documentation.

---

## Check Your Understanding

Let's see if you've got this. Here are a couple of questions to test your understanding.

### Question 1

You're modeling a user registration flow for a social media app. Which scenario best represents a BDD-style "Given-When-Then" structure?

**A)**
```sruja
// partial
RegistrationFlow = scenario "User Registration" {
  Customer -> WebApp "Registers"
  WebApp -> API "Saves user"
  API -> Database "Persists"
}
```

**B)**
```sruja
// partial
RegistrationFlow = scenario "As a new user, I want to register so I can use the app" {
  // GIVEN: User is on registration page
  Customer -> WebApp "Views registration form"
  
  // WHEN: User submits valid registration
  Customer -> WebApp "Enters email and password"
  Customer -> WebApp "Clicks 'Sign Up'"
  WebApp -> API "Submits registration"
  
  // THEN: Account is created and user is logged in
  API -> Database "Creates user account"
  API -> WebApp "Returns success with session token"
  WebApp -> Customer "Shows 'Welcome! Redirecting to dashboard...'"
}
```

**C)**
```sruja
// partial
RegistrationFlow = scenario "Registration Process" {
  Start -> Database "Create user"
  Database -> Email "Send welcome"
}
```

**D)**
```sruja
// partial
RegistrationFlow = story "User Registration Story" {
  WebApp -> API "Register"
  API -> Database "Save"
}
```

<details>
<summary>Click to see the answer</summary>

**Answer: B) BDD-style "Given-When-Then" scenario**

Let's analyze each option:

**A)** Incorrect. This scenario is too abstract. It shows the technical steps (registers, saves, persists) but doesn't capture the user's perspective or experience. There's no "Given-When-Then" structure—it's just a sequence of technical actions. This reads more like a data flow than a user journey.

**B)** Correct! This is a perfect BDD-style user journey because:
- **Title**: "As a new user, I want to register so I can use the app" follows the user story format
- **GIVEN**: "User is on registration page" — sets up the starting state
- **WHEN**: "User submits valid registration" — describes the user action
- **THEN**: "Account is created and user is logged in" — describes the expected outcome
- **User's perspective**: Shows what the user sees (form, button, welcome message, redirect)
- **Complete experience**: From viewing the form to being logged in

This scenario serves multiple purposes:
- **Requirements**: It's unambiguous what "register" means
- **Tests**: It can be turned directly into a test case
- **Documentation**: Anyone reading understands the user experience
- **Communication**: Product managers, developers, and testers all agree on what "register" means

**C)** Incorrect. This is far too abstract. "Start → Database → Email" tells you nothing about the user. What does the user do? What do they see? What happens from their perspective? This looks more like a data flow (and a poor one at that) than a user journey. It's missing the most important part—the human user.

**D)** Incorrect. While this uses the `story` keyword (which is fine), it's far too simple to be useful. "Register → Save" doesn't tell you:
- What the user actually does (enters email? clicks button?)
- What the system shows (success page? error message?)
- What the experience is (how long does it take? what do they see?)
- What happens if something goes wrong

This scenario is so vague it provides no real value. A BDD scenario should be detailed enough that anyone—developer, tester, product manager—understands exactly what happens.

**Key insight:** BDD-style scenarios focus on the user's experience, not just technical steps. They follow a clear "Given-When-Then" structure that makes requirements unambiguous. A good scenario should be detailed enough that it can serve as both requirements documentation and a test case.

</details>

---

### Question 2

You're modeling a login flow and want to document an error path. Which scenario best models a meaningful error handling experience?

**A)**
```sruja
// partial
LoginError = scenario "Login Error" {
  User -> WebApp "Login with wrong password"
  WebApp -> API "Authenticate"
  API -> Database "Check password"
  Database -> API "Password doesn't match"
  API -> WebApp "Return error"
  WebApp -> User "Shows error"}
```

**B)**
```sruja
// partial
LoginError = scenario "Login Error" {
  User -> WebApp "Login with wrong password"
  WebApp -> API "Authenticate"
  API -> Database "Check password"
  Database -> API "Password doesn't match"
  API -> WebApp "Return error: AUTH_FAILED"
  WebApp -> User "Shows error: 'Authentication failed'"}
```

**C)**
```sruja
// partial
LoginError = scenario "Login with Incorrect Password" {
  User -> WebApp "Enters email: user@example.com and wrong password"
  User -> WebApp "Clicks 'Log In'"
  WebApp -> API "Submits login"
  API -> Database "Verifies credentials"
  Database -> API "Password doesn't match"
  API -> WebApp "Returns error: Invalid credentials"
  WebApp -> User "Shows 'Password incorrect. Please try again or reset your password if you've forgotten it.' with link to password reset"}
```

**D)**
```sruja
// partial
LoginError = scenario "Login Failed" {
  API -> Database "Check"
  Database -> API "No match"
  API -> WebApp "Error"
  WebApp -> User "Can't login"}
```

<details>
<summary>Click to see the answer</summary>

**Answer: C) Shows user action, system response, and helpful error message**

Let's analyze each option:

**A)** Incorrect. While this scenario shows the error occurring, it's missing crucial details:
- It shows the user action ("Login with wrong password") but doesn't show the specific form interaction (enters email? clicks button?)
- It shows "Return error" but doesn't specify what the error is
- It shows "Shows error" but doesn't tell you what error message the user sees
- It doesn't help the user understand what to do next

The error message is cryptic—"Shows error" tells the user nothing. What kind of error? What should they do? Try again? Reset password? Contact support?

**B)** Incorrect. This is better than option A but still has problems:
- The error code "AUTH_FAILED" is technical and cryptic to users
- The error message "Authentication failed" is vague and unhelpful
- It doesn't guide the user on what to do next
- It doesn't offer alternatives (password reset, contact support)

Users seeing this error will be confused: "Authentication failed? What does that mean? Is my email wrong? My password? My account locked? Should I try again? Reset my password?"

**C)** Correct! This scenario models an excellent error handling experience because:
- **User action is specific**: User enters email AND wrong password AND clicks "Log In" — shows the complete action
- **System response is clear**: API returns "Invalid credentials" — specific, not generic
- **User experience is helpful**: Error message is "Password incorrect. Please try again or reset your password if you've forgotten it." — tells the user what's wrong and what they can do about it
- **Provides alternatives**: Links to password reset if they've forgotten it
- **User's perspective**: Shows what the user actually sees and experiences

This error message follows best practices:
- **Specific**: Tells you exactly what's wrong (password, not email or account)
- **Actionable**: Tells you what you can do (try again or reset password)
- **Helpful**: Provides a link to password reset
- **Human**: Not "ERR_AUTH_403" or "Authentication failed"

**D)** Incorrect. This scenario is a mess:
- It starts with "API" instead of showing the user action — this is from the system's perspective, not the user's
- "Check" and "No match" are meaningless labels — they don't tell you what's actually happening
- "Error" is vague — what kind of error?
- "Can't login" is the user's experience, but it doesn't help them understand why or what to do

This scenario is too abstract to be useful. It doesn't capture the user's perspective, doesn't provide helpful information, and doesn't guide next steps.

**Key insight:** Error paths should be just as well-designed as happy paths. When modeling errors:
- Be specific about what went wrong (not just "error occurred")
- Write helpful error messages (not cryptic codes)
- Guide the user on next steps (try again? reset password? contact support?)
- Provide alternatives when appropriate (password reset link, different payment method, etc.)
- Think from the user's perspective (what do they see? what do they understand? what do they do next?)

A well-designed error path turns a frustrating experience into a helpful one. I once saw error messages drop support tickets by 70% just by making them more helpful and actionable.

</details>

---

## What's Next?

Congratulations! You've completed Module 4: Flows. You now understand:

- What flows are and how they differ from static relationships
- How to create data flow diagrams that show lineage and transformations
- How to model user journeys that capture the complete user experience
- How to document both happy paths and error paths
- How to use BDD-style scenarios for requirements and testing

You can now create diagrams that tell complete stories—stories about how data moves through your system, and stories about how users experience your system. You can model data lineage, transformations, bottlenecks, and user journeys.

In the next module, you'll learn about **feedback loops**—how systems regulate themselves through circular cause-and-effect relationships. You'll discover how positive feedback loops amplify change and negative feedback loops stabilize systems. You'll learn to recognize these patterns in real systems and understand their powerful effects.

See you there!

---

## Module 4 Complete!

You've now mastered the art of modeling flows. Here's what you've learned:

**Lesson 1: Understanding Flows**
- Flows show sequence and transformation, not just connections
- Different types: data flows, user journeys, control flows, event flows
- Use flows when order matters, when you need to see bottlenecks

**Lesson 2: Data Flow Diagrams**
- Data flows show lineage: where data comes from and where it goes
- Document transformations: how data changes shape at each step
- Common patterns: ETL pipelines, event sourcing, real-time analytics, lambda architecture

**Lesson 3: User Journeys**
- User journeys show the complete user experience from their perspective
- Model both happy paths and error paths
- Use BDD-style "Given-When-Then" for clarity
- Document edge cases and unusual scenarios

You're ready to tackle more advanced concepts. Let's continue!
