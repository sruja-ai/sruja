contextMap {
  workflows.pipeline -> repos.repoCore: "customer-supplier"
  workflows.pipeline -> runners.agent: "upstream"
  runners.agent -> artifacts.storage: "customer-supplier"
  billing.usage -> workflows.pipeline: "upstream"
}