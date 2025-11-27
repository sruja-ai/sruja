module authService {
  context: identity.auth
  owner: team.platform

  container api: Service "Auth API"
  container db: Database "User Store"

  api -> db: "validate credentials"
}