import "../issue-service.dsl" as issues

module analyticsService {
  context: analytics.stats
  owner: team.analytics

  container api: Service "Analytics API"
  api -> issues.api
}