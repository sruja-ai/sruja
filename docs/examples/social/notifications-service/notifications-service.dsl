import "../feed-service.dsl" as feed

module notificationService {
  context: notifications.notify
  owner: team.notifications

  container api: Service "Notification API"
  api -> feed.api
}