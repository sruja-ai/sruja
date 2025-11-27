module inventoryService {
  context: inventory.stock
  owner: team.inventory

  container api: Service "Inventory API"
  api -> api: "internal"
}