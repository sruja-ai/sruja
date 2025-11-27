import "../checkout-service/architecture.dsl" as checkout

module paymentService {
  context: payments.corepay
  owner: team.payments

  container api: Service "Payment API"
  container db: Database "Payment DB"

  api -> db: "charge"
}