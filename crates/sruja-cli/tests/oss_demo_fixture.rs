//! CI guard for pinned OSS demo structural drift output.

use std::path::PathBuf;
use std::process::Command;

fn sruja_bin() -> PathBuf {
    let bin_name = format!("sruja{}", std::env::consts::EXE_SUFFIX);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("debug")
        .join(bin_name)
}

#[test]
fn minimal_structural_drift_matches_golden_envelope() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let minimal = root.join("examples/oss-demo/minimal");
    let golden_path = root.join("examples/oss-demo/minimal-structural-drift.json");
    if !minimal.exists() {
        eprintln!(
            "skip: oss-demo fixture directory missing: {}",
            minimal.display()
        );
        return;
    }
    if !golden_path.exists() {
        eprintln!(
            "skip: oss-demo golden file missing: {}",
            golden_path.display()
        );
        return;
    }
    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&golden_path).expect("read golden"))
            .expect("parse golden");

    let bin = sruja_bin();
    if !bin.exists() {
        eprintln!("skip: debug sruja binary not built");
        return;
    }

    let out = Command::new(&bin)
        .arg("drift")
        .arg("-r")
        .arg(&minimal)
        .arg("-f")
        .arg("json")
        .arg("--structural-only")
        .arg("--advisory")
        .output()
        .expect("run drift");
    assert!(
        out.status.success(),
        "drift failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let live: serde_json::Value = serde_json::from_slice(&out.stdout).expect("parse drift json");

    assert_eq!(
        live.get("clean_scan").and_then(|v| v.as_bool()),
        golden.get("clean_scan").and_then(|v| v.as_bool()),
        "clean_scan mismatch"
    );
    assert!(
        live.get("could_not_infer")
            .map(|v| v.is_array())
            .unwrap_or(false),
        "missing could_not_infer"
    );
    assert!(live.get("scan_scope").is_some(), "missing scan_scope");
}
