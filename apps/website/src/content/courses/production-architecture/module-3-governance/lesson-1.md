---
title: "Lesson 1: Interview Question - Design a HIPAA-Compliant Healthcare System"
weight: 1
summary: "Answer compliance and governance questions for senior-level interviews."
---

# Lesson 1: Interview Question - Design a HIPAA-Compliant Healthcare System

## The Interview Question

**"Design a healthcare platform that stores patient data and must comply with HIPAA regulations. How do you ensure compliance across all services?"**

This is a **senior/staff level interview question** that tests:

- Understanding of compliance requirements
- Ability to enforce standards at scale
- Security and privacy considerations
- Governance and policy enforcement

## Step 1: Clarify Requirements

**You should ask:**

- "What are the core features? Patient records, appointments, prescriptions?"
- "What's the scale? How many patients, healthcare providers?"
- "What compliance requirements? HIPAA, SOC 2, others?"
- "What about data retention? How long must we keep records?"

**Interviewer's answer:**

- "Core: Patient records, appointments, prescriptions, billing"
- "Scale: 10M patients, 100K healthcare providers"
- "Must comply with HIPAA (health data privacy)"
- "Retain records for 10 years (legal requirement)"

## Step 2: Understand HIPAA Requirements

**Key HIPAA requirements** (you should mention these):

1. **Encryption**: Data at rest and in transit
2. **Access Control**: Role-based access, audit logs
3. **Audit Logging**: Track all access to patient data
4. **Data Minimization**: Only collect necessary data
5. **Breach Notification**: Report breaches within 72 hours

## Step 3: Design with Policies

This is where Sruja's `policy` feature is perfect! Show how you enforce compliance:

```sruja
element person
element system
element container
element component
element datastore
element queue

// HIPAA Compliance Policy
policy HIPAACompliance "All patient data must be encrypted and access logged" {
category "compliance"
enforcement "required"
description "HIPAA requires encryption at rest and in transit, plus audit logging for all patient data access"
}

// Security Policy
policy TLSEnforcement "All external communications must use TLS 1.3" {
category "security"
enforcement "required"
description "Required for HIPAA compliance - all data in transit must be encrypted"
}

policy EncryptionAtRest "All patient data must be encrypted at rest using AES-256" {
category "security"
enforcement "required"
description "HIPAA requirement - database encryption, file encryption"
}

// Access Control Policy
policy AccessControl "Role-based access control required for all patient data" {
category "security"
enforcement "required"
description "Only authorized healthcare providers can access patient data"
}

// Audit Logging Policy
policy AuditLogging "All access to patient data must be logged" {
category "compliance"
enforcement "required"
description "HIPAA requires audit trails - who accessed what, when, why"
}

// Observability Policy
policy Observability "All services must expose health check and metrics endpoints" {
category "observability"
enforcement "required"
metadata {
  healthEndpoint "/health"
  metricsEndpoint "/metrics"
}
}

HealthcareApp = system "Healthcare Application" {
PatientAPI = container "Patient API" {
  technology "Go, gRPC"
  tags ["encrypted", "audit-logged"]
  description "Handles patient data - must comply with HIPAACompliance policy"
}

AppointmentAPI = container "Appointment API" {
  technology "Java, Spring Boot"
  tags ["encrypted"]
  description "Manages appointments - must comply with all policies"
}

BillingAPI = container "Billing API" {
  technology "Node.js, Express"
  tags ["encrypted", "audit-logged"]
  description "Handles billing - contains PHI (Protected Health Information)"
}

PatientDB = datastore "Patient Database" {
  technology "PostgreSQL"
  tags ["encrypted", "audit-logged"]
  description "Encrypted at rest, all access logged for HIPAA compliance"
}

AuditLogDB = datastore "Audit Log Database" {
  technology "PostgreSQL"
  description "Stores audit logs - immutable, append-only"
}

AuditQueue = queue "Audit Log Queue" {
  technology "Kafka"
  description "Async audit logging to avoid blocking operations"
}
}

IdentityProvider = system "Identity Provider" {
tags ["external"]
description "OAuth2/OIDC for authentication and authorization"
}

// Show compliance in action
HealthcareApp.PatientAPI -> HealthcareApp.PatientDB "Reads/Writes (encrypted, logged)"
HealthcareApp.PatientAPI -> HealthcareApp.AuditLogDB "Logs access via AuditQueue"
HealthcareApp.PatientAPI -> IdentityProvider "Validates access tokens"

view index {
include *
}
```

## What Interviewers Look For

### ✅ Good Answer (What You Just Did)

1. **Understood compliance requirements** - Mentioned specific HIPAA rules
2. **Defined policies explicitly** - Showed governance thinking
3. **Applied policies to architecture** - Tags, descriptions show compliance
4. **Addressed security** - Encryption, access control, audit logging
5. **Explained enforcement** - How policies are enforced

### ❌ Bad Answer (Common Mistakes)

1. Not understanding compliance requirements
2. No mention of policies or governance
3. Ignoring security (encryption, access control)
4. No audit logging strategy
5. Can't explain how to enforce standards

## Key Points to Mention in Interview

### 1. Policy-Driven Architecture

**Say**: "I define policies at the architecture level to enforce standards. For example:

- HIPAACompliance policy requires encryption and audit logging
- All services that handle patient data must comply
- Policies are checked in CI/CD - non-compliant services can't deploy"

### 2. Encryption Strategy

**Say**: "We encrypt data at multiple levels:

- **In transit**: TLS 1.3 for all communications
- **At rest**: AES-256 encryption for databases
- **Application level**: Encrypt sensitive fields before storing"

### 3. Access Control

**Say**: "We use:

- **OAuth2/OIDC**: For authentication and authorization
- **Role-based access control (RBAC)**: Doctors can access their patients, admins have broader access
- **Principle of least privilege**: Users only get minimum required access"

### 4. Audit Logging

**Say**: "We log all access to patient data:

- **What**: Which patient record was accessed
- **Who**: Which user/role accessed it
- **When**: Timestamp
- **Why**: Purpose of access (treatment, billing, etc.)
- **Immutable logs**: Can't be modified or deleted"

### 5. Enforcement Strategy

**Say**: "We enforce policies through:

- **CI/CD checks**: Validate architecture before deployment
- **Service mesh policies**: Enforce TLS, rate limiting
- **Database policies**: Encryption at rest, access controls
- **Monitoring**: Alert on policy violations"

## Interview Practice: Add More Compliance

**Interviewer might ask**: "What about data retention and deletion?"

Add data retention policy:

```sruja
element person
element system
element container
element component
element datastore
element queue

// Existing policies (HIPAACompliance, TLSEnforcement, etc. from main design)
policy HIPAACompliance "All patient data must be encrypted and access logged" {
category "compliance"
enforcement "required"
}

// Additional policies
policy DataRetention "Patient records retained for 10 years, then archived" {
category "compliance"
enforcement "required"
description "Legal requirement - records must be retained for 10 years, then moved to cold storage"
}

policy RightToDeletion "Support patient right to data deletion (with exceptions)" {
category "compliance"
enforcement "required"
description "GDPR/HIPAA - patients can request data deletion, but some data must be retained for legal reasons"
}

HealthcareApp = system "Healthcare Application" {
PatientAPI = container "Patient API" {
  technology "Go, gRPC"
  tags ["encrypted", "audit-logged"]
}

PatientDB = datastore "Patient Database" {
  technology "PostgreSQL"
  description "Active patient records - 10 year retention"
}

ArchiveDB = datastore "Archive Database" {
  technology "S3 Glacier"
  description "Cold storage for records older than 10 years"
}
}

view index {
include *
}
```

## Common Follow-Up Questions

Be prepared for:

1. **"How do you ensure all services comply?"**
   - Answer: "Policy validation in CI/CD. Architecture review process. Service mesh enforces some policies automatically. Regular audits."

2. **"What if a service violates a policy?"**
   - Answer: "CI/CD blocks deployment. Alert security team. Architecture review required. Service owner must fix before deploying."

3. **"How do you handle breaches?"**
   - Answer: "Automated breach detection via monitoring. Incident response plan. HIPAA requires notification within 72 hours. Audit logs help identify scope."

4. **"How do you balance compliance with developer productivity?"**
   - Answer: "Automate compliance checks. Provide templates and libraries. Make compliance easy, not burdensome. Clear documentation and examples."

## Exercise: Practice This Question

Design a HIPAA-compliant healthcare system and be ready to explain:

1. How you enforce HIPAA requirements
2. Your encryption strategy
3. Your access control approach
4. Your audit logging implementation
5. How you ensure compliance across services

**Practice tip**: This is a senior-level question. Focus on:

- Governance and policies
- Security and compliance
- Enforcement strategies
- Trade-offs and practical considerations

## Key Takeaways for Senior Interviews

1. **Understand compliance requirements** - Know HIPAA, SOC 2, GDPR basics
2. **Define policies explicitly** - Show governance thinking
3. **Enforce at multiple levels** - CI/CD, service mesh, monitoring
4. **Balance compliance and productivity** - Make it easy for developers
5. **Think about scale** - How to enforce across 100+ services

## Next Steps

You've learned how to handle compliance and governance questions. This completes the Production Architecture course! You're now ready to tackle:

- ✅ Scaling and performance questions
- ✅ Microservices architecture questions
- ✅ Senior-level governance questions

**Keep practicing with real interview questions!** 🎯
