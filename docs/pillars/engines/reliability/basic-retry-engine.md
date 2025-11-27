# Basic Retry Engine

**Status**: Core Engine  
**Pillars**: Reliability

[← Back to Engines](../README.md)

## Overview

The Basic Retry Engine provides retry policy validation, ensuring services define and validate retry mechanisms for resilience.

**This provides basic retry capabilities for reliability.**

## Purpose

The Basic Retry Engine:

- ✅ Validates retry policy definitions
- ✅ Checks retry configuration
- ✅ Validates retry strategies
- ✅ Provides retry recommendations
- ✅ Supports basic resilience

## Retry Policy Types

### Fixed Retry
- Fixed retry count
- Fixed retry interval
- Fixed retry validation
- Fixed retry configuration

### Exponential Backoff
- Exponential retry count
- Exponential retry interval
- Exponential retry validation
- Exponential retry configuration

### Linear Backoff
- Linear retry count
- Linear retry interval
- Linear retry validation
- Linear retry configuration

## Validation Rules

### Policy Validation
- Retry policy exists
- Retry policy is configured
- Retry policy is valid
- Retry policy is consistent

### Configuration Validation
- Retry count is valid
- Retry interval is valid
- Backoff strategy is valid
- Timeout values are valid

## Integration Points

### Retry Policy Engine (Advanced)
- Extends basic functionality
- Provides advanced features

### Failure Propagation Engine (FPE)
- Uses retry policies
- Models retry behavior

### Validation Engine
- Uses validation rules
- Provides error reporting

## MCP API

```
retry.basic.validate(service)
retry.basic.policy(service)
retry.basic.recommend(service)
```

## Strategic Value

The Basic Retry Engine provides:

- ✅ Retry policy validation
- ✅ Basic resilience
- ✅ Failure handling
- ✅ Service reliability

**This is essential for basic reliability.**

## Implementation Status

✅ Architecture designed  
✅ Retry policy types specified  
✅ Integration points defined  
📋 Implementation in progress

---

*The Basic Retry Engine provides retry policy validation.*

