import "inventory-service.dsl" as inv
import "billing-service.dsl" as bill

module bookingService {
  context: booking.bookingCore
  owner: team.booking

  container api: Service "Booking API"

  api -> inv.api
  api -> bill.api
}