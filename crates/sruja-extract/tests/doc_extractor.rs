mod common;
use common::*;
use sruja_extract::docs::DocExtractor;

#[test]
fn doc_extractor_name() {
    let extractor = DocExtractor::new();
    assert_eq!(extractor.name(), "docs");
}

#[test]
fn doc_extractor_detects_markdown() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("README.md");
    fs::write(&file_path, "# Test Project\nSome content").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::Readme);
}

#[test]
fn doc_extractor_detects_docs() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("CHANGELOG.md");
    fs::write(&file_path, "# Changes").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].binding.kind, SourceKind::Docs);
}

#[test]
fn doc_extractor_detects_rst() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("doc.rst");
    fs::write(&file_path, "Document").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn doc_extractor_detects_asciidoc() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("guide.adoc");
    fs::write(&file_path, "= Guide Title\nContent").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
}

#[test]
fn doc_extractor_extracts_title_from_markdown() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("guide.md");
    fs::write(&file_path, "# My Architecture Guide\nContent").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].binding.description.as_deref(),
        Some("My Architecture Guide")
    );
}

#[test]
fn doc_extractor_extracts_title_from_asciidoc() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("guide.adoc");
    fs::write(&file_path, "= AsciiDoc Title\nContent").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert_eq!(
        results[0].binding.description.as_deref(),
        Some("AsciiDoc Title")
    );
}

#[test]
fn doc_extractor_ignores_non_doc_files() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("script.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(results.is_empty());
}

#[test]
fn doc_extractor_includes_relative_path() {
    let tmp = temp_dir();
    let file_path = tmp.path().join("docs").join("guide.md");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "# Guide").unwrap();

    let results = check(&DocExtractor::new(), &file_path, tmp.path());
    assert!(!results.is_empty());
    assert!(results[0].binding.path.contains("docs/guide.md"));
}

#[test]
fn doc_readme_higher_confidence_than_other_docs() {
    let tmp = temp_dir();

    let readme = tmp.path().join("README.md");
    fs::write(&readme, "# Readme").unwrap();
    let readme_results = check(&DocExtractor::new(), &readme, tmp.path());

    let changelog = tmp.path().join("notes.md");
    fs::write(&changelog, "# Notes").unwrap();
    let changelog_results = check(&DocExtractor::new(), &changelog, tmp.path());

    assert!(readme_results[0].confidence > changelog_results[0].confidence);
}
