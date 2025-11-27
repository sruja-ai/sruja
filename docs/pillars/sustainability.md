# Sustainability Pillar

This document describes how Sruja supports the Sustainability pillar of the Well-Architected Framework.

[← Back to Pillars Index](./README.md)

## Overview

**Sustainability** focuses on minimizing environmental impacts, efficient use of resources, and shared responsibility for sustainability in cloud architectures.

---

## Core Support (Basic)

The core DSL includes basic sustainability features:

### Resource Efficiency

```sruja
system MyService {
  sustainability {
    resource_efficiency: "high"
  }
}
```

### Basic Carbon Tracking

```sruja
system MyService {
  carbon_footprint {
    estimated_monthly: "50 kg CO2"
  }
}
```

---

## Advanced Extensions

### Sustainability DSL

Complete sustainability modeling:

```sruja
sustainability {
  carbon_footprint APIServiceCarbon {
    compute: "30 kg CO2/month"
    storage: "10 kg CO2/month"
    network: "5 kg CO2/month"
    total: "45 kg CO2/month"
  }

  resource_efficiency {
    cpu_utilization: "target > 60%"
    memory_utilization: "target > 70%"
    storage_efficiency: "compression enabled"
  }

  green_computing {
    use_renewable_energy: true
    region_preference: ["us-west-2", "eu-west-1"]  // Regions with renewable energy
    carbon_neutral: true
  }

  sustainability_goals {
    carbon_reduction_target: "-20% by 2025"
    energy_efficiency: "improve 15% annually"
  }
}
```

### Resource Optimization

```sruja
sustainability {
  optimization {
    right_sizing {
      current: "over-provisioned"
      recommended: "c5.large"
      savings: "40% compute, 30% carbon"
    }

    idle_resource_cleanup {
      enabled: true
      timeout: "24h"
    }

    data_lifecycle {
      hot_data: "SSD, 30 days"
      warm_data: "HDD, 90 days"
      cold_data: "archive, 1 year"
    }
  }
}
```

### Carbon Accounting

```sruja
carbon_accounting {
  service APIService {
    compute_carbon: "30 kg CO2/month"
    storage_carbon: "10 kg CO2/month"
    network_carbon: "5 kg CO2/month"
  }

  region_breakdown {
    us_east_1: "20 kg CO2/month"
    eu_west_1: "15 kg CO2/month"
  }

  trends {
    month_over_month: "-5%"
    target: "-20% by 2025"
  }
}
```

### Green Architecture Patterns

```sruja
green_patterns {
  serverless_first {
    rationale: "Pay only for execution time"
    carbon_savings: "30-50%"
  }

  regional_optimization {
    use_renewable_regions: true
    minimize_data_transfer: true
  }

  efficient_algorithms {
    caching: "reduce compute by 40%"
    batch_processing: "reduce API calls by 60%"
  }
}
```

---

## Integration with Other Pillars

### Cost Optimization
- Sustainability often aligns with cost savings
- Efficient resources = lower cost + lower carbon

### Performance Efficiency
- Efficient performance = lower resource usage = lower carbon

### Operational Excellence
- Sustainable operations
- Green DevOps practices

---

## Validation Rules

Sustainability validation includes:

- ✅ Carbon footprint must be tracked
- ✅ Resource efficiency must be optimized
- ✅ Renewable energy preferences should be set
- ✅ Sustainability goals must be defined
- ✅ Idle resources should be cleaned up

---

## Engines

### Core Engines
- **Basic Resource Efficiency Engine** - Resource efficiency validation
- **Basic Carbon Tracking Engine** - Carbon footprint tracking

### Advanced Engines
- **Sustainability Engine** - Full sustainability DSL support
- **Carbon Accounting Engine** - Carbon footprint calculation
- **Resource Efficiency Engine** - CPU, memory, storage efficiency
- **Green Computing Engine** - Renewable energy, carbon-neutral
- **Sustainability Goals Engine** - Carbon reduction targets
- **Idle Resource Cleanup Engine** - Automatic resource cleanup
- **Data Lifecycle Engine** - Hot/warm/cold data management

See: [Engines by Pillars](./engines.md#sustainability-pillar) for complete list

---

## DSL Reference

See: [DSL Extensions](../specs/dsl-extensions.md) - Sustainability section (to be added)

---

## Examples

- [Sustainability Example](../examples/sustainability/)
- [Green Architecture](../examples/green-architecture/)

---

*Sustainability ensures your architecture minimizes environmental impact while delivering business value.*

