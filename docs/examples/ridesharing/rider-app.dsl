import "../auth-service.dsl" as auth
import "../dispatch-service.dsl" as dispatch

module riderApp {
  context: mobility.rider
  owner: team.rider

  container ui: Frontend "Rider App"
  ui -> auth.api: "login"
  ui -> dispatch.api: "request ride"
}