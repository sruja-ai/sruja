import "../booking-service.dsl" as booking

module reviewService {
  context: reviews.feedback
  owner: team.reviews

  container api: Service "Review API"
  api -> booking.api
}