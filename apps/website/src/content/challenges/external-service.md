---
title: "Weather App: Integrate Weather API"
summary: "Your weather app needs real-time weather data! Integrate an external weather service (like OpenWeatherMap) that your API can query for current conditions."
difficulty: intermediate
topic: integration
estimatedTime: "10-15 min"
initialDsl: |
  architecture "Weather App" {
    person User "App User"
    
    system WeatherApp {
      container MobileApp "Mobile App"
      container WeatherAPI "Weather Service API"
      datastore UserPrefs "User Preferences DB"
    }
    
    User -> MobileApp "Checks weather"
    MobileApp -> WeatherAPI "Requests forecast"
    WeatherAPI -> UserPrefs "Loads preferences"
    
    // TODO: Add external WeatherService and connect WeatherAPI -> WeatherService
  }
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
  - "External services are defined outside system blocks using 'external' keyword"
  - "Use: external WeatherService \"OpenWeatherMap\""
  - "External services represent third-party APIs you don't control"
  - "Add relation: WeatherAPI -> WeatherService \"Fetches weather data\""
solution: |
  architecture "Weather App" {
    person User "App User"
    
    system WeatherApp {
      container MobileApp "Mobile App"
      container WeatherAPI "Weather Service API"
      datastore UserPrefs "User Preferences DB"
    }
    
    external WeatherService "OpenWeatherMap"
    
    User -> MobileApp "Checks weather"
    MobileApp -> WeatherAPI "Requests forecast"
    WeatherAPI -> UserPrefs "Loads preferences"
    WeatherAPI -> WeatherService "Fetches weather data"
  }
---
