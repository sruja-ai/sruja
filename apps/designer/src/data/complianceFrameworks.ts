// apps/designer/src/data/complianceFrameworks.ts

/**
 * Compliance frameworks and their requirements.
 */
export interface ComplianceRequirement {
  id: string;
  description: string;
  category: "access-control" | "encryption" | "audit" | "data-protection" | "network" | "other";
  severity: "critical" | "high" | "medium" | "low";
}

export interface ComplianceFramework {
  id: "SOC2" | "HIPAA" | "PCI-DSS" | "GDPR" | "ISO27001";
  name: string;
  description: string;
  requirements: ComplianceRequirement[];
}

/**
 * Pre-defined compliance frameworks.
 */
export const COMPLIANCE_FRAMEWORKS: ComplianceFramework[] = [
  {
    id: "SOC2",
    name: "SOC 2",
    description:
      "Service Organization Control 2 - Security, availability, processing integrity, confidentiality, and privacy",
    requirements: [
      {
        id: "SOC2-CC1",
        description: "Control environment - Management establishes oversight",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "SOC2-CC2",
        description: "Communication and information - System communicates relevant information",
        category: "data-protection",
        severity: "high",
      },
      {
        id: "SOC2-CC3",
        description: "Risk assessment - System identifies and assesses risks",
        category: "other",
        severity: "high",
      },
      {
        id: "SOC2-CC4",
        description: "Monitoring activities - System monitors controls",
        category: "audit",
        severity: "high",
      },
      {
        id: "SOC2-CC5",
        description: "Control activities - System implements controls",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "SOC2-CC6",
        description: "Logical and physical access controls",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "SOC2-CC7",
        description: "System operations - System operations are authorized",
        category: "other",
        severity: "high",
      },
    ],
  },
  {
    id: "HIPAA",
    name: "HIPAA",
    description: "Health Insurance Portability and Accountability Act - Healthcare data protection",
    requirements: [
      {
        id: "HIPAA-164.308",
        description: "Administrative safeguards - Security management process",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "HIPAA-164.312",
        description: "Technical safeguards - Access control, audit controls, integrity",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "HIPAA-164.314",
        description: "Organizational requirements - Business associate agreements",
        category: "other",
        severity: "high",
      },
      {
        id: "HIPAA-Encryption",
        description: "Encryption of ePHI in transit and at rest",
        category: "encryption",
        severity: "critical",
      },
      {
        id: "HIPAA-Audit",
        description: "Audit logs for access to ePHI",
        category: "audit",
        severity: "critical",
      },
    ],
  },
  {
    id: "PCI-DSS",
    name: "PCI DSS",
    description: "Payment Card Industry Data Security Standard - Payment card data protection",
    requirements: [
      {
        id: "PCI-1",
        description: "Install and maintain firewall configuration",
        category: "network",
        severity: "critical",
      },
      {
        id: "PCI-2",
        description: "Do not use vendor-supplied defaults",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "PCI-3",
        description: "Protect stored cardholder data",
        category: "data-protection",
        severity: "critical",
      },
      {
        id: "PCI-4",
        description: "Encrypt transmission of cardholder data",
        category: "encryption",
        severity: "critical",
      },
      {
        id: "PCI-7",
        description: "Restrict access to cardholder data",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "PCI-10",
        description: "Track and monitor network access",
        category: "audit",
        severity: "critical",
      },
    ],
  },
  {
    id: "GDPR",
    name: "GDPR",
    description: "General Data Protection Regulation - EU data protection",
    requirements: [
      {
        id: "GDPR-Art32",
        description: "Security of processing - Appropriate technical measures",
        category: "encryption",
        severity: "critical",
      },
      {
        id: "GDPR-Art33",
        description: "Notification of personal data breach",
        category: "data-protection",
        severity: "high",
      },
      {
        id: "GDPR-Art35",
        description: "Data protection impact assessment",
        category: "data-protection",
        severity: "high",
      },
    ],
  },
  {
    id: "ISO27001",
    name: "ISO 27001",
    description: "Information Security Management System",
    requirements: [
      {
        id: "ISO-A.9",
        description: "Access control",
        category: "access-control",
        severity: "critical",
      },
      {
        id: "ISO-A.10",
        description: "Cryptography",
        category: "encryption",
        severity: "critical",
      },
      {
        id: "ISO-A.12",
        description: "Operations security",
        category: "other",
        severity: "high",
      },
    ],
  },
];

/**
 * Get a compliance framework by ID.
 */
export function getComplianceFramework(
  id: ComplianceFramework["id"]
): ComplianceFramework | undefined {
  return COMPLIANCE_FRAMEWORKS.find((f) => f.id === id);
}

/**
 * Get all compliance frameworks.
 */
export function getAllComplianceFrameworks(): ComplianceFramework[] {
  return COMPLIANCE_FRAMEWORKS;
}
