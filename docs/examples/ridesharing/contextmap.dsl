contextMap {
  mobility.rider -> identity.auth: "customer-supplier"
  mobility.dispatch -> pricing.priceEngine: "conformist"
  mobility.dispatch -> location.geoservice: "customer-supplier"
  payments.billing -> pricing.priceEngine: "conformist"
}