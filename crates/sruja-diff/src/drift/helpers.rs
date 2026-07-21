/// Path patterns that usually indicate doc, test, or tooling code rather than main product.
/// Orphans and god modules in these paths are excluded so the score reflects product code only.
pub(crate) fn is_likely_doc_or_tool_path(path: &str, id: &str) -> bool {
    let p = path.replace('\\', "/").to_lowercase();
    let id_lower = id.to_lowercase();
    p.ends_with("doc.go")
        || p.contains("/doc/")
        || p.contains("_test.go")
        || p.contains("/test/")
        || p.contains("/tests/")
        || p.contains("__tests__")
        || p.contains(".spec.")
        || p.contains(".test.")
        || p.contains("/tools/")
        || p.contains("/vendor/")
        || p.contains("/third_party/")
        || p.contains("/deps/")
        || p.contains("node_modules/")
        || p.contains("/stories/")
        || p.contains(".stories.")
        || p.contains("/examples/")
        || p.contains("/fixtures/")
        || p.contains("/sample")
        || p.contains("/mocks/")
        || p.contains("/mock/")
        || p.contains(".config.")
        || p.contains("config/")
        || p.contains("/scripts/")
        || p.contains("/build/")
        || p.contains("/migrations/")
        || p.contains("/setup/")
        || p.ends_with("webpack.config.js")
        || p.ends_with("vite.config.ts")
        || p.ends_with("jest.config.js")
        || p.ends_with(".eslintrc.js")
        || p.ends_with("tailwind.config.js")
        || id_lower.contains("_doc_go")
        || id_lower.ends_with("_test_go")
        || id_lower.contains("_config_")
        || id_lower.contains("_mock_")
        || id_lower.contains("_stories_")
}

/// Paths that are commonly entry points or re-export hubs; reporting them as orphans
/// is usually a false positive (scanner may not see dynamic requires or re-exports).
pub fn is_likely_entry_point(path: &str, id: &str) -> bool {
    let p = path.replace('\\', "/");
    let p_lower = p.to_lowercase();
    let id_lower = id.to_lowercase();

    if id_lower.starts_with("module:") && id_lower.contains("_src_commands") {
        return true;
    }
    if p_lower.ends_with("index.js")
        || p_lower.ends_with("index.ts")
        || p_lower.ends_with("index.jsx")
        || p_lower.ends_with("index.tsx")
        || p_lower.ends_with("main.js")
        || p_lower.ends_with("main.ts")
        || p_lower.ends_with("app.js")
        || p_lower.ends_with("app.ts")
        || p_lower.ends_with("main.rs")
        || p_lower.ends_with("lib.rs")
        || p_lower.ends_with("main.py")
        || p_lower.ends_with("__init__.py")
        || p_lower.ends_with("main.go")
        || p_lower.ends_with("mod.rs")
    {
        return !p_lower.contains("/examples/")
            && !p_lower.contains("/tests/")
            && !p_lower.contains("/test/")
            && !p_lower.contains("_test.");
    }
    if p_lower.ends_with(".js") && p_lower.contains("/lib/") {
        let after_lib = p_lower.split("/lib/").last().unwrap_or("");
        if !after_lib.contains('/') {
            return true;
        }
    }
    if p_lower.ends_with("/src/lib.rs") {
        return true;
    }
    false
}

/// Files that are typically consumed through framework mechanisms
/// (decorators, DI, reflection) rather than direct imports.
pub(crate) fn is_likely_framework_consumed(path: &str, id: &str) -> bool {
    let p = path.replace('\\', "/").to_lowercase();
    let id_lower = id.to_lowercase();

    p.ends_with(".decorator.ts")
        || p.ends_with(".guard.ts")
        || p.ends_with(".pipe.ts")
        || p.ends_with(".interceptor.ts")
        || p.ends_with(".filter.ts")
        || p.ends_with(".middleware.ts")
        || p.ends_with(".strategy.ts")
        || p.ends_with(".module.ts")
        || p.ends_with(".transformer.ts")
        || p.ends_with(".enum.ts")
        || p.ends_with(".enum.js")
        || p.ends_with(".type.ts")
        || p.ends_with(".types.ts")
        || p.ends_with(".dto.ts")
        || p.ends_with(".entity.ts")
        || p.ends_with(".interface.ts")
        || p.ends_with(".constants.ts")
        || p.ends_with("/apps.py")
        || p.ends_with("/admin.py")
        || p.ends_with("/signals.py")
        || id_lower.contains("configuration")
        || id_lower.contains("interceptor")
}

pub(crate) fn top_targets_for_module(graph: &sruja_scan::Graph, module_id: &str, n: usize) -> Vec<String> {
    let mut targets: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.source == module_id)
        .map(|e| e.target.clone())
        .collect();
    targets.sort();
    targets.truncate(n);
    targets
}
