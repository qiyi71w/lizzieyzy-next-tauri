use std::path::PathBuf;

fn frontend_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src")
}

#[test]
fn analysis_cache_product_commands_are_not_registered() {
    let source = include_str!("../src/lib.rs");
    for name in [
        ["compute", "game", "cache", "key"].join("_"),
        ["get", "analysis", "cache"].join("_"),
        ["save", "analysis", "cache"].join("_"),
        ["delete", "analysis", "cache"].join("_"),
    ] {
        if source.contains(&name) {
            panic!("{name} remains in the desktop gateway");
        }
    }
}

#[test]
fn analysis_cache_frontend_modules_are_absent() {
    let src = frontend_src();
    for relative in [
        "api/analysisCache.ts",
        "domain/cache.ts",
        "components/CacheStatusBadge.tsx",
    ] {
        let path = src.join(relative);
        if path.exists() {
            panic!("{} still exists", path.display());
        }
    }
}
