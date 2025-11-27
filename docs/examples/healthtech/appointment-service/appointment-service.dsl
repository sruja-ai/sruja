import "../patient-service.dsl" as pat

module appointmentService {
  context: appointments.schedule
  owner: team.appt

  container api: Service "Appointment API"
  api -> pat.api
}