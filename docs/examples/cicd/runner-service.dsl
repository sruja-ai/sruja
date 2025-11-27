import "artifact-service.dsl" as art

module runnerService {
  context: runners.agent
  owner: team.runners

  container api: Service "Runner API"
  api -> art.api
}