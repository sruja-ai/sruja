mod common;
use common::*;
use sruja_extract::dockerfile::DockerfileExtractor;

#[test]
fn dockerfile_extractor_name() {
    assert_eq!(DockerfileExtractor::new().name(), "dockerfile");
}

#[test]
fn dockerfile_extractor_detects_dockerfile() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Dockerfile");
    fs::write(
        &file_path,
        "FROM node:20-alpine\nWORKDIR /app\nCOPY . .\nEXPOSE 3000\nCMD [\"node\", \"server.js\"]\n",
    )
    .unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::Dockerfile);
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(desc.contains("node:20-alpine"), "should contain base image");
    assert!(desc.contains("3000"), "should contain port");
}

#[test]
fn dockerfile_extractor_detects_variants() {
    let tmp = temp_dir();
    for name in &[
        "Dockerfile.prod",
        "Dockerfile.dev",
        "app.dockerfile",
        "Containerfile",
    ] {
        let file_path = tmp.path().join(name);
        fs::write(&file_path, "FROM ubuntu:22.04\nRUN apt-get update\n").unwrap();

        let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
        assert!(!results.is_empty(), "should detect {name}");
    }
}

#[test]
fn dockerfile_extractor_ignores_non_dockerfile() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Makefile");
    fs::write(&file_path, "FROM = something\nall: build\n").unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn dockerfile_extractor_suggests_parent_dir() {
    let tmp = temp_dir();
    let dir = tmp.path().join("payment-service");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("Dockerfile");
    fs::write(&file_path, "FROM golang:1.22\n").unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert_eq!(
        results[0].suggested_element.as_deref(),
        Some("payment-service")
    );
}

#[test]
fn dockerfile_extractor_no_ports() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("Dockerfile");
    fs::write(&file_path, "FROM python:3.12\nRUN pip install flask\n").unwrap();

    let results = check(&DockerfileExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    let desc = results[0].binding.description.as_deref().unwrap();
    assert!(desc.contains("python:3.12"));
    assert!(!desc.contains("ports"));
}
