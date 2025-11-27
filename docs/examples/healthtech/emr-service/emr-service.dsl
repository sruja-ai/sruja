import "../patient-service.dsl" as pat
import "../lab-service.dsl" as labs

module emrService {
  context: records.emr
  owner: team.records

  container api: Service "EMR API"
  api -> pat.api
  api -> labs.api
}