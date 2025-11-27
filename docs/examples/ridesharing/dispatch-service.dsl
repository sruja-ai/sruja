import "../pricing-service.dsl" as price
import "../geoservice.dsl" as geo

module dispatchService {
  context: mobility.dispatch
  owner: team.dispatch

  container api: Service "Dispatch API"
  api -> price.api: "get price"
  api -> geo.api: "get ETA"
}