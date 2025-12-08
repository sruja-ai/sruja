---
title: "Lesson 1: Load Balancers"
weight: 1
summary: "L4 vs L7 Load Balancing, Algorithms."
---

# Lesson 1: Load Balancers

## What is a Load Balancer?

A load balancer sits between clients and servers, distributing incoming network traffic across a group of backend servers. This ensures that no single server bears too much load.

## Types of Load Balancing

### Layer 4 (Transport Layer)
*   Decisions based on IP address and TCP/UDP ports.
*   Faster, less CPU intensive.
*   Does not inspect the content of the request.

### Layer 7 (Application Layer)
*   Decisions based on the content of the message (URL, HTTP headers, cookies).
*   Can route traffic to different services based on URL (e.g., `/images` to image servers).
*   More CPU intensive but smarter.

## Algorithms

*   **Round Robin:** Requests are distributed sequentially.
*   **Least Connections:** Sends request to the server with the fewest active connections.
*   **IP Hash:** The client's IP address is used to determine which server receives the request (useful for session stickiness).

---

## 🛠️ Sruja Perspective: Modeling Load Balancers

In Sruja, a load balancer is typically modeled as a `container` or `component` that sits in front of your application servers.

```sruja
architecture "Web Application" {
    container LB "Nginx Load Balancer" {
        technology "Nginx"
        tags "load-balancer"
        description "Layer 7 load balancer routing traffic based on URL paths."
    }

    container AppServer "App Server" {
        technology "Python, Django"
        tags "scaled"
    }

    // Traffic flow
    LB -> AppServer "Distributes requests (Round Robin)"
}
```
