import "../follow-service.dsl" as follow
import "../media-service.dsl" as media

module feedService {
  context: timeline.feed
  owner: team.timeline

  container api: Service "Feed API"
  api -> follow.api
  api -> media.api
}