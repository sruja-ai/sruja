module authService {
  context: identity.auth
  owner: team.identity

  container api: Service "Auth API"
  container db: Database "User DB"
  api -> db
}