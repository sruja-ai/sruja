---
title: "Weather App: Integrate Weather API"
summary: "Your weather app needs real-time weather data! Integrate an external weather service (like OpenWeatherMap) that your API can query for current conditions."
difficulty: intermediate
topic: integration
estimatedTime: "10-15 min"
initialDsl: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  database = kind "Database"

  User = person "App User"

  WeatherApp = system "Weather App" {
    MobileApp = container "Mobile App"
    WeatherAPI = container "Weather Service API"
    UserPrefs = database "User Preferences DB"
  }

  User -> WeatherApp.MobileApp "Checks weather"
  WeatherApp.MobileApp -> WeatherApp.WeatherAPI "Requests forecast"
  WeatherApp.WeatherAPI -> WeatherApp.UserPrefs "Loads preferences"

  // TODO: Add external WeatherService system with tags and connect WeatherAPI -> WeatherService
checks:
  - type: noErrors
    message: "DSL parsed successfully"
  - type: elementExists
    name: WeatherService
    message: "Add external WeatherService"
  - type: relationExists
    source: WeatherAPI
    target: WeatherService
    message: "Connect WeatherAPI -> WeatherService"
hints:
  - 'External services are represented as systems with tags ["external"]'
  - 'Use: WeatherService = system "OpenWeatherMap" { tags ["external"] }'
  - "External services represent third-party APIs you don't control"
  - 'Add relation: WeatherApp.WeatherAPI -> WeatherService "Fetches weather data"'
solution: |
  person = kind "Person"
  system = kind "System"
  container = kind "Container"
  database = kind "Database"

  User = person "App User"

  WeatherApp = system "Weather App" {
    MobileApp = container "Mobile App"
    WeatherAPI = container "Weather Service API"
    UserPrefs = database "User Preferences DB"
  }

  WeatherService = system "OpenWeatherMap" {
    tags ["external"]
  }

  User -> WeatherApp.MobileApp "Checks weather"
  WeatherApp.MobileApp -> WeatherApp.WeatherAPI "Requests forecast"
  WeatherApp.WeatherAPI -> WeatherApp.UserPrefs "Loads preferences"
  WeatherApp.WeatherAPI -> WeatherService "Fetches weather data"
---
