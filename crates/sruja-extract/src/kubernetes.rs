//! Extractor for Kubernetes manifest files.
//!
//! Detects Deployments, Services, StatefulSets, DaemonSets, Jobs, CronJobs,
//! Ingresses, ConfigMaps, HPAs, and other common resource types.

use crate::utils::yaml;
use crate::{DiscoveredSource, ExtractError, Extractor, FileContext};
use sruja_language::ast::{SourceBinding, SourceKind};

const K8S_RESOURCE_KINDS: &[&str] = &[
    "Deployment",
    "Service",
    "StatefulSet",
    "DaemonSet",
    "Job",
    "CronJob",
    "Ingress",
    "ConfigMap",
    "Secret",
    "HorizontalPodAutoscaler",
    "PersistentVolumeClaim",
    "NetworkPolicy",
    "ServiceAccount",
    "Role",
    "ClusterRole",
    "RoleBinding",
    "ClusterRoleBinding",
    "Namespace",
    "Pod",
    "ReplicaSet",
];

const WORKLOAD_KINDS: &[&str] = &[
    "Deployment",
    "StatefulSet",
    "DaemonSet",
    "Job",
    "CronJob",
    "Pod",
    "ReplicaSet",
];

#[derive(Default)]
pub struct KubernetesExtractor;

impl KubernetesExtractor {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn parse_k8s_resources(content: &str) -> Vec<(String, String)> {
        yaml::parse_yaml_resources(content, K8S_RESOURCE_KINDS)
    }

    pub(crate) fn is_workload(kind: &str) -> bool {
        WORKLOAD_KINDS.contains(&kind)
    }

    pub(crate) fn confidence_for_kind(kind: &str) -> f32 {
        if Self::is_workload(kind) {
            0.8
        } else {
            0.6
        }
    }
}

impl Extractor for KubernetesExtractor {
    fn name(&self) -> &'static str {
        "kubernetes"
    }

    fn check_file(&self, ctx: &FileContext) -> Result<Vec<DiscoveredSource>, ExtractError> {
        let ext = ctx.extension().to_lowercase();
        if !matches!(ext.as_str(), "yaml" | "yml") {
            return Ok(Vec::new());
        }

        let content = match ctx.content() {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        if !content.contains("apiVersion:") {
            return Ok(Vec::new());
        }

        let resources = Self::parse_k8s_resources(content);
        if resources.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        for (kind, name) in resources {
            let confidence = Self::confidence_for_kind(&kind);

            results.push(DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::Kubernetes,
                    path: ctx.relative_path().to_string(),
                    description: Some(format!("Kubernetes {kind}: {name}")),
                },
                suggested_element: if Self::is_workload(&kind)
                    || kind == "Service"
                    || kind == "Ingress"
                {
                    Some(name)
                } else {
                    None
                },
                confidence,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_workload() {
        assert!(KubernetesExtractor::is_workload("Deployment"));
        assert!(KubernetesExtractor::is_workload("StatefulSet"));
        assert!(KubernetesExtractor::is_workload("DaemonSet"));
        assert!(KubernetesExtractor::is_workload("Job"));
        assert!(KubernetesExtractor::is_workload("CronJob"));
        assert!(KubernetesExtractor::is_workload("Pod"));
    }

    #[test]
    fn test_is_not_workload() {
        assert!(!KubernetesExtractor::is_workload("Service"));
        assert!(!KubernetesExtractor::is_workload("ConfigMap"));
        assert!(!KubernetesExtractor::is_workload("Ingress"));
        assert!(!KubernetesExtractor::is_workload("Namespace"));
    }

    #[test]
    fn test_confidence_for_kind_workload() {
        assert_eq!(KubernetesExtractor::confidence_for_kind("Deployment"), 0.8);
        assert_eq!(KubernetesExtractor::confidence_for_kind("Job"), 0.8);
    }

    #[test]
    fn test_confidence_for_kind_non_workload() {
        assert_eq!(KubernetesExtractor::confidence_for_kind("ConfigMap"), 0.6);
        assert_eq!(KubernetesExtractor::confidence_for_kind("Secret"), 0.6);
    }

    #[test]
    fn test_parse_k8s_resources_basic() {
        let content = "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: api\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: api-svc";
        let resources = KubernetesExtractor::parse_k8s_resources(content);
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0], ("Deployment".to_string(), "api".to_string()));
        assert_eq!(resources[1], ("Service".to_string(), "api-svc".to_string()));
    }

    #[test]
    fn test_parse_k8s_resources_filters_unknown_kind() {
        let content = "apiVersion: v1\nkind: UnknownKind\nmetadata:\n  name: test";
        let resources = KubernetesExtractor::parse_k8s_resources(content);
        assert!(resources.is_empty());
    }

    #[test]
    fn test_parse_k8s_resources_multi_document() {
        let content = "---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: api\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: api-svc\n---\napiVersion: apps/v1\nkind: StatefulSet\nmetadata:\n  name: db";
        let resources = KubernetesExtractor::parse_k8s_resources(content);
        assert_eq!(resources.len(), 3);
        assert_eq!(resources[2], ("StatefulSet".to_string(), "db".to_string()));
    }
}
