import "repo-service.dsl" as repo
import "runner-service.dsl" as runner

module workflowService {
  context: workflows.pipeline
  owner: team.workflows

  container api: Service "Workflow Engine"
  api -> repo.api
  api -> runner.api
}