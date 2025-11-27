import "project-service.dsl" as proj

module issueService {
  context: issues.issueTrack
  owner: team.issues

  container api: Service "Issue API"
  api -> proj.api
}