import "../auth-service/architecture.dsl" as auth

module accountsService {
  context: accounts.accountCore
  owner: team.accounts

  container api: Service "Accounts API"
  api -> auth.api: "authenticate"
}