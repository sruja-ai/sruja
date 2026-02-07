---
title: "Lesson 1: Introduction to the Project"
weight: 1
summary: "Defining the scope of our Shopify-lite platform."
---

# Lesson 1: Introduction to the Project

In this course, we are building **Shopify-lite**. Let's define what that means.

## The Concept
We are building a **multi-tenant e-commerce platform**. This means a single instance of our software will serve multiple different online stores (tenants), each with their own products, orders, and customers.

## Core Capabilities
Our system must support:
1.  **Storefronts**: Fast, SEO-friendly pages for browsing products.
2.  **Admin Dashboard**: Where merchants manage their inventory.
3.  **Checkout**: A secure, reliable way to take money.
4.  **Inventory Management**: Real-time stock tracking to prevent overselling.

## The "Why"
Why build this? Because it touches on every hard problem in distributed systems:
*   **Consistency**: Inventory must be accurate.
*   **Availability**: Checkout must never go down.
*   **Scalability**: We need to handle flash sales.
*   **Security**: We are handling credit card data (PCI Compliance).

## The Role of Sruja
Most tutorials start by running `npx create-next-app`. **We will not do that yet.**

We will start by creating a `sruja` file. Why? Because we need to agree on the *structure* before we get lost in the *details*. Sruja will be our shared whiteboard, our documentation, and our validator.
