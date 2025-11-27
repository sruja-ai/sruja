import "../issue-service.dsl" as issues

module notificationService {
  context: notifications.notify
  owner: team.notifications

  container api: Service "Notification API"
  api -> issues.api
}