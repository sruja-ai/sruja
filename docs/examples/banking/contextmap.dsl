contextMap {
  payments.transfers -> risk.fraud: "customer-supplier"
  payments.transfers -> compliance.aml: "customer-supplier"
  cards.cardsIssuer -> accounts.accountCore: "upstream"
  accounts.accountCore -> identity.auth: "conformist"
}