import "../inventory-service/architecture.dsl" as inv
import "../logistics-service/architecture.dsl" as ship

module orderService {
  context: ordering.orderMgmt
  owner: team.orders

  container api: Service "Order API"

  api -> inv.api: "update reservation"
  api -> ship.api: "create shipment"
}