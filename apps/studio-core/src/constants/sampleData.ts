// apps/studio-core/src/constants/sampleData.ts
import type { ArchitectureJSON } from '@sruja/viewer';

export const SAMPLE_JSON: ArchitectureJSON = {
  "metadata": {
    "name": "C4 Complete Example",
    "version": "1.0.0",
    "generated": "2025-12-01T15:49:57+05:30"
  },
  "architecture": {
    "systems": [
      {
        "id": "WebApp",
        "label": "Web Application",
        "containers": [
          {
            "id": "API",
            "label": "API Service"
          },
          {
            "id": "DB",
            "label": "Database"
          }
        ]
      }
    ],
    "persons": [
      {
        "id": "User",
        "label": "End User"
      }
    ],
    "relations": [
      {
        "from": "User",
        "to": "WebApp.API",
        "verb": "Credentials"
      },
      {
        "from": "WebApp.API",
        "to": "WebApp.DB",
        "verb": "Validate User"
      }
    ]
  },
  "navigation": {
    "levels": [
      "level1",
      "level2",
      "level3"
    ]
  }
};









