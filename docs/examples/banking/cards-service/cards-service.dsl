import "../accounts-service/architecture.dsl" as acc

module cardsService {
  context: cards.cardsIssuer
  owner: team.cards

  container api: Service "Cards API"
  api -> acc.api: "fetch owner"
}