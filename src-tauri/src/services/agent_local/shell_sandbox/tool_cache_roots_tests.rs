use super::*;

#[test]
fn defaults_follow_platform_cache_layouts() {
    let home = Path::new("/home/user");
    let mac_cache = Path::new("/home/user/Library/Caches");
    let linux_cache = Path::new("/home/user/.cache");
    let windows_cache = Path::new("/home/user/AppData/Local");

    let mac = collect_for(
        Platform::Macos,
        home,
        Some(mac_cache),
        &CacheOverrides::default(),
    );
    let linux = collect_for(
        Platform::Linux,
        home,
        Some(linux_cache),
        &CacheOverrides::default(),
    );
    let windows = collect_for(
        Platform::Windows,
        home,
        Some(windows_cache),
        &CacheOverrides::default(),
    );

    for roots in [&mac, &linux, &windows] {
        assert_eq!(roots.len(), MAX_WRITE_DIRS);
        assert!(roots.contains(&home.join(".cargo/registry")));
        assert!(roots.contains(&home.join(".cargo/git")));
        assert!(roots.contains(&home.join("go/pkg/mod")));
        assert!(roots.contains(&home.join("go/pkg/sumdb")));
    }
    assert!(mac.contains(&mac_cache.join("pip")));
    assert!(mac.contains(&mac_cache.join("go-build")));
    assert!(mac.contains(&mac_cache.join("Yarn")));
    assert!(mac.contains(&home.join(".cache/uv")));
    assert!(linux.contains(&linux_cache.join("pip")));
    assert!(linux.contains(&linux_cache.join("go-build")));
    assert!(linux.contains(&linux_cache.join("yarn")));
    assert!(windows.contains(&windows_cache.join("npm-cache/_cacache")));
    assert!(windows.contains(&windows_cache.join("pip/Cache")));
    assert!(windows.contains(&windows_cache.join("uv/cache")));
    assert!(windows.contains(&windows_cache.join("Yarn/Cache")));
}

#[test]
fn standard_environment_overrides_replace_every_default() {
    let home = Path::new("/home/user");
    let overrides = CacheOverrides {
        cargo_home: Some(home.join("custom/cargo")),
        npm_cache: Some(home.join("custom/npm")),
        pip_cache: Some(home.join("custom/pip")),
        uv_cache: Some(home.join("custom/uv")),
        go_build: Some(home.join("custom/go-build")),
        yarn_cache: Some(home.join("custom/yarn")),
        go_modules: Some(home.join("custom/go-modules")),
        go_path: Some(home.join("ignored-go-path")),
        xdg_cache: Some(home.join("ignored-xdg")),
    };

    let roots = collect_for(Platform::Linux, home, None, &overrides);

    assert_eq!(roots.len(), MAX_WRITE_DIRS);
    assert!(roots.contains(&home.join("custom/cargo/registry")));
    assert!(roots.contains(&home.join("custom/cargo/git")));
    assert!(roots.contains(&home.join("custom/npm/_cacache")));
    assert!(roots.contains(&home.join("custom/pip")));
    assert!(roots.contains(&home.join("custom/uv")));
    assert!(roots.contains(&home.join("custom/go-build")));
    assert!(roots.contains(&home.join("custom/yarn")));
    assert!(roots.contains(&home.join("custom/go-modules")));
    assert!(roots.contains(&home.join("ignored-go-path/pkg/sumdb")));
}
