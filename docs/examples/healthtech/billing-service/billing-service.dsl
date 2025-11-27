import "../emr-service.dsl" as emr

module billingService {
  context: billing.claims
  owner: team.billing

  container api: Service "Billing API"
  api -> emr.api
}