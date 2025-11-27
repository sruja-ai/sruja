import "../auth-service/architecture.dsl" as auth
import "../payment-service/architecture.dsl" as pay
import "../inventory-service/architecture.dsl" as inv

module checkoutService {
  context: ordering.checkout
  owner: team.checkout

  container api: Service "Checkout API"

  api -> auth.api: "validate user"
  api -> pay.api: "charge customer"
  api -> inv.api: "reserve stock"
}