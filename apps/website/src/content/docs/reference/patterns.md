---
title: "Architecture Patterns"
weight: 2
summary: "Reusable patterns: request/response, event-driven, saga, CQRS."
---

# Architecture Patterns

## Request/Response

```sruja
system App {
  container Web
  container API
  datastore DB
}
App.Web -> App.API "Calls"
App.API -> App.DB "Reads/Writes"
```

## Event-Driven

```sruja
system Orders {
  container OrderSvc
  container PaymentSvc
}
Orders.OrderSvc -> Orders.PaymentSvc "OrderCreated event"
Orders.PaymentSvc -> Orders.OrderSvc "PaymentConfirmed event"
```

## Saga

```sruja
scenario CreateOrderSaga {
  Orders.OrderSvc -> Orders.InventorySvc "Reserve"
  Orders.InventorySvc -> Orders.OrderSvc "Reserved"
  Orders.OrderSvc -> Orders.PaymentSvc "Charge"
  Orders.PaymentSvc -> Orders.OrderSvc "Charged"
}
```

## CQRS

```sruja
system App {
  container CommandAPI
  container QueryAPI
  datastore ReadDB
  datastore WriteDB
}
App.CommandAPI -> App.WriteDB "Writes"
App.QueryAPI -> App.ReadDB "Reads"
```
