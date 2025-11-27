import "../messaging-service.dsl" as msg

module notificationService {
  context: notifications.notify
  owner: team.notifications

  container api: Service "Notification Service"
  api -> msg.api
}