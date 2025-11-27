contextMap {
  appointments.schedule -> patients.patientCore: "upstream"
  records.emr -> labs.results: "customer-supplier"
  billing.claims -> records.emr: "upstream"
}