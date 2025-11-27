import "../messaging-service.dsl" as msg

module mediaService {
  context: media.mediaStore
  owner: team.media

  container api: Service "Media Store"
  api -> msg.api
}