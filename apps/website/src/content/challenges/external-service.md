---
title: "Weather App: Integrate Weather API"
summary: "Your weather app needs real-time weather data! Integrate an external weather service (like OpenWeatherMap) that your API can query for current conditions."
difficulty: intermediate
topic: integration
estimatedTime: "10-15 min"
initialDsl: |
  specification {
    element person
    element system
    element container
    element component
    element datastore
    element queue
    element external
  }
  
  model {
    User = person "App User"
    
      WeatherApp = system  {
        MobileApp = container "Mobile App"
        WeatherAPI = container "Weather Service API"
        UserPrefs = datastore "User Preferences DB"
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
  specification {
    element person
    element system
    element container
    element component
    element datastore
    element queue
    element external
  }
  
  model {
    User = person "App User"
    
      WeatherApp = system  {
        MobileApp = container "Mobile App"
        WeatherAPI = container "Weather Service API"
        UserPrefs = datastore "User Preferences DB"
      }
    
      WeatherService = external "OpenWeatherMap"
    
      User -> MobileApp "Checks weather"
      MobileApp -> WeatherAPI "Requests forecast"
      WeatherAPI -> UserPrefs "Loads preferences"
      WeatherAPI -> WeatherService "Fetches weather data"
    
  }
---
