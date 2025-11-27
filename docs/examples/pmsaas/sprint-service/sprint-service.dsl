import "../issue-service.dsl" as issues

module sprintService {
  context: sprints.sprintPlan
  owner: team.sprints

  container api: Service "Sprints API"
  api -> issues.api
}