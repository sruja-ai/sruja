#![allow(clippy::await_holding_lock)]
use super::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

static ENV_LOCK: OnceLock<std::sync::Mutex<()>> = OnceLock::new();

mod initialize;
mod resources;
mod tools;
mod watch;

fn write_mcp_fixture_repo(dir: &std::path::Path) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).expect("create src");
    std::fs::write(src.join("lib.rs"), "pub fn api() {}\n").expect("write lib");
    std::fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "mcp-fixture"
version = "0.1.0"
edition = "2021"
"#,
    )
    .expect("write cargo");
    std::fs::write(
        dir.join("repo.sruja"),
        r#"
system = kind "System"
component = kind "Component"

App = system "App" {
  description "App"

  Svc = component "Service" {
    description "Service"
    state_machine "Lifecycle" {
      initial "Created"
      terminal ["Done"]
      "Created" -> "Done" on "finish"
    }
    contract "Get" {
      input { id "string" }
      output { ok "bool" }
      error { "ERR" "failed" }
    }
  }
}
"#,
    )
    .expect("write dsl");
}
