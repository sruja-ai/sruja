//! Extractor for Kubernetes manifest files.
//!
//! Detects Deployments, Services, StatefulSets, DaemonSets, Jobs, CronJobs,
//! Ingresses, ConfigMaps, HPAs, and other common resource types.

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

pub struct KubernetesExtractor;

impl Default for KubernetesExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl KubernetesExtractor {
    pub fn new() -> Self {
        Self
    }

    fn parse_k8s_resources(content: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        let mut current_kind: Option<String> = None;
        let mut current_name: Option<String> = None;
        let mut in_metadata = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "---" {
                if let (Some(kind), Some(name)) = (current_kind.take(), current_name.take()) {
                    results.push((kind, name));
                }
                in_metadata = false;
                continue;
            }

            if let Some(kind_str) = trimmed.strip_prefix("kind:") {
                let kind = kind_str.trim().trim_matches('"').to_string();
                if K8S_RESOURCE_KINDS.contains(&kind.as_str()) {
                    current_kind = Some(kind);
                }
            } else if trimmed == "metadata:" {
                in_metadata = true;
            } else if in_metadata && trimmed.starts_with("name:") {
                let indent = line.len() - line.trim_start().len();
                if indent <= 4 {
                    current_name = Some(
                        trimmed["name:".len()..]
                            .trim()
                            .trim_matches('"')
                            .to_string(),
                    );
                    in_metadata = false;
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let indent = line.len() - line.trim_start().len();
                if indent == 0 && trimmed != "apiVersion:" && !trimmed.starts_with("apiVersion:") {
                    in_metadata = false;
                }
            }
        }

        if let (Some(kind), Some(name)) = (current_kind, current_name) {
            results.push((kind, name));
        }

        results
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
            let is_workload = WORKLOAD_KINDS.contains(&kind.as_str());
            let confidence = if is_workload { 0.8 } else { 0.6 };

            results.push(DiscoveredSource {
                binding: SourceBinding {
                    kind: SourceKind::Kubernetes,
                    path: ctx.relative_path().to_string(),
                    description: Some(format!("Kubernetes {kind}: {name}")),
                },
                suggested_element: if is_workload || kind == "Service" || kind == "Ingress" {
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
