contextMap {
  search.searchEngine -> inventory.hotelInventory: "customer-supplier"
  booking.bookingCore -> payments.billing: "upstream"
  reviews.feedback -> booking.bookingCore: "upstream"
}