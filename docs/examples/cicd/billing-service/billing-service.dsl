import "../workflow-service.dsl" as wf

module billingService {
  context: billing.usage
  owner: team.billing

  container api: Service "Billing API"
  api -> wf.api
}