import "../cards-service.dsl" as cards
import "../fraud-service.dsl" as fraud
import "../aml-service.dsl" as aml

module paymentsService {
  context: payments.transfers
  owner: team.payments

  container api: Service "Payments API"
  api -> fraud.api
  api -> aml.api
}