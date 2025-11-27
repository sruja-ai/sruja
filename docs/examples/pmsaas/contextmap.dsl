contextMap {
  issues.issueTrack -> projects.projectCore: "upstream"
  analytics.stats -> issues.issueTrack: "customer-supplier"
  notifications.notify -> issues.issueTrack: "customer-supplier"
  sprints.sprintPlan -> issues.issueTrack: "upstream"
}