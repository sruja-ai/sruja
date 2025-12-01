---
title: "Lesson 2: Setting up the Workspace"
weight: 2
summary: "Initializing the project structure and creating the first architecture file."
---

# Lesson 2: Setting up the Workspace

Let's get our hands dirty. We will set up a professional project structure that separates our architectural definitions from our implementation code.

## 1. Directory Structure

Create a new directory for your project:

```bash
mkdir shopify-lite
cd shopify-lite
```

We will use the following structure:

```text
shopify-lite/
├── architecture/       # Sruja files live here
│   ├── main.sruja
│   └── ...
├── src/                # Source code (Go, Node, etc.)
├── docs/               # Generated documentation
└── README.md
```

## 2. Installing Sruja

If you haven't already, install the Sruja CLI:

```bash
go install github.com/sruja-ai/sruja/cmd/sruja@latest
```

## 3. Hello World: The Context View

Create your first file at `architecture/main.sruja`. We'll start with a high-level **Context View** to define the boundaries of our system.

```sruja
architecture "Shopify-Lite" {
    
    // 1. The System
    system Platform "E-Commerce Platform" {
        description "The core multi-tenant e-commerce engine."
    }

    // 2. The Users
    person Merchant "Store Owner"
    person Shopper "Customer"

    // 3. External Systems
    system Stripe "Payment Gateway" {
        external
    }

    // 4. High-Level Interactions
    Merchant -> Platform "Manages Store"
    Shopper -> Platform "Browses & Buys"
    Platform -> Stripe "Processes Payments"
}
```

## 4. Visualize It

Run the Sruja CLI to visualize your architecture:

```bash
sruja view architecture/main.sruja
```

You should see a clean diagram showing your platform sitting between your users and the payment gateway. This is our starting point.
