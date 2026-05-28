//! Additional export format integration tests.

mod common;
use common::{create_test_repo, run_sruja, write_file};

const MINIMAL_VALID_SRUJA: &str = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" {
  description "End user"
}

App = system "My App" {
  description "Main application"

  Web = container "Web" {
    technology "React"
    description "UI"
  }
}
User -> App "uses"
"#;

#[test]
fn export_markdown_succeeds_on_valid_file() {
    let repo = create_test_repo();
    write_file(repo.path(), "arch.sruja", MINIMAL_VALID_SRUJA);
    let path_str = repo
        .path()
        .join("arch.sruja")
        .to_str()
        .expect("utf-8")
        .to_string();

    let (success, stdout, stderr) = run_sruja(&["export", "markdown", &path_str]);

    assert!(success, "export markdown should succeed: {stderr}");
    assert!(
        stdout.contains('#') || stdout.contains("App") || stdout.contains("User"),
        "markdown export should include headings or element names: stdout={stdout}"
    );
}
