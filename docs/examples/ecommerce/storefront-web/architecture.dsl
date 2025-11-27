import "../auth-service/architecture.dsl" as auth

module storefrontWeb {
  context: ordering.checkout
  owner: team.checkout

  container web: Frontend "Storefront Web App"

  web -> auth.api: "login"
}