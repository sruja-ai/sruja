import api_gateway
import database

def render():
    # BAD PATTERN: bypass gateway
    return database.query()

// Simulated behavior:
console.log('simulated code load');
