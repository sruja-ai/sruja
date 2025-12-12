---
title: "Examples"
weight: 80
summary: "Full, runnable examples of Sruja architectures."
---

# Examples

Here are some complete examples of Sruja models. You can edit and run them directly in the playground.

## Banking System

A simplified version of the C4 model banking system example.

```sruja
architecture "Banking System" {
    system BankingSystem "Internet Banking System" {
        description "Allows customers to view accounts and make payments."

        container WebApp "Web Application" {
            technology "Java and Spring MVC"
            description "Delivers the static content and the Internet banking single page application."
        }

        container Database "Database" {
            technology "Oracle Database"
            description "Stores user registration information, hashed credentials, etc."
        }

        BankingSystem.WebApp -> BankingSystem.Database "Reads from and writes to"
    }

    person Customer "Personal Banking Customer" {
        description "A customer of the bank, with personal bank accounts."
    }

    Customer -> BankingSystem.WebApp "Visits"
}
```

## Microservices

An example showing a microservices architecture.

```sruja
architecture "E-Commerce" {
    system Shop "Online Shop" {
        container Frontend "Storefront" {
            technology "Next.js"
        }

        container Catalog "Catalog Service" {
            technology "Go"
        }

        container Orders "Order Service" {
            technology "Java"
        }

        datastore DB "Main DB" {
            technology "PostgreSQL"
        }

        Shop.Frontend -> Shop.Catalog "Browses items"
        Shop.Frontend -> Shop.Orders "Places orders"
        Shop.Catalog -> Shop.DB "Reads products"
        Shop.Orders -> Shop.DB "Writes orders"
    }

    person Shopper "Shopper"

    Shopper -> Shop.Frontend "Browses and buys"
}
```
