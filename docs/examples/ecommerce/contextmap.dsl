contextMap {
  identity.auth -> ordering.checkout: "customer-supplier"
  ordering.checkout -> payments.corepay: "conformist"
  ordering.orderMgmt -> inventory.stock: "conformist"
  inventory.stock -> logistics.fulfillment: "customer-supplier"
}