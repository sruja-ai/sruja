mod common;
use common::*;
use sruja_extract::kubernetes::KubernetesExtractor;

#[test]
fn kubernetes_extractor_name() {
    assert_eq!(KubernetesExtractor::new().name(), "kubernetes");
}

#[test]
fn kubernetes_extractor_detects_deployment() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("deployment.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: payment-service\nspec:\n  replicas: 3\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("payment-service")
    );
}

#[test]
fn kubernetes_extractor_detects_service() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("service.yaml");
    fs::write(
        &file_path,
        "apiVersion: v1\nkind: Service\nmetadata:\n  name: user-api\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(results[0].suggested_element.as_deref(), Some("user-api"));
}

#[test]
fn kubernetes_extractor_detects_statefulset() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("statefulset.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: StatefulSet\nmetadata:\n  name: database\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(results[0].suggested_element.as_deref(), Some("database"));
}

#[test]
fn kubernetes_extractor_detects_daemonset() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("daemonset.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: DaemonSet\nmetadata:\n  name: log-collector\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("log-collector")
    );
}

#[test]
fn kubernetes_extractor_detects_cronjob() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("cronjob.yaml");
    fs::write(
        &file_path,
        "apiVersion: batch/v1\nkind: CronJob\nmetadata:\n  name: nightly-cleanup\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn kubernetes_extractor_detects_ingress() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("ingress.yaml");
    fs::write(
        &file_path,
        "apiVersion: networking.k8s.io/v1\nkind: Ingress\nmetadata:\n  name: api-gateway\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert_eq!(results[0].suggested_element.as_deref(), Some("api-gateway"));
}

#[test]
fn kubernetes_extractor_multi_document_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("all.yaml");
    fs::write(
        &file_path,
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: svc-a\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: svc-b\n",
    )
    .unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 2);
}

#[test]
fn kubernetes_extractor_ignores_non_k8s_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.yaml");
    fs::write(&file_path, "setting: value\napiVersion: v1").unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn kubernetes_extractor_ignores_non_yaml() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("config.json");
    fs::write(&file_path, "{\"apiVersion\": \"v1\"}").unwrap();

    let results = check(&KubernetesExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn kubernetes_workloads_higher_confidence() {
    let tmp = temp_dir();
    let deploy = tmp.path().join("deploy.yaml");
    fs::write(
        &deploy,
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: app\n",
    )
    .unwrap();
    let deploy_results = check(&KubernetesExtractor::new(), &deploy, tmp.path());

    let configmap = tmp.path().join("cm.yaml");
    fs::write(
        &configmap,
        "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: settings\n",
    )
    .unwrap();
    let cm_results = check(&KubernetesExtractor::new(), &configmap, tmp.path());

    assert!(deploy_results[0].confidence > cm_results[0].confidence);
}
