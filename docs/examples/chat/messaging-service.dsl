import "channel-service.dsl" as channel
import "search-service.dsl" as search

module messagingService {
  context: messaging.core
  owner: team.messaging

  container api: Service "Messaging API"

  api -> channel.api
  api -> search.api
}