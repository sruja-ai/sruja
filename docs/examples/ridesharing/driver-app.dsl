import "../dispatch-service.dsl" as dispatch

module driverApp {
  context: mobility.driver
  owner: team.driver

  container ui: Frontend "Driver App"
  ui -> dispatch.api: "get matched"
}