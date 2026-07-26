use std::path::PathBuf;

pub fn python_path() -> Result<Option<PathBuf>, String> {
    if !cfg!(windows) {
        return Ok(None);
    }

    let dir = super::paths::sidecar_dir().join("compat");
    std::fs::create_dir_all(&dir).map_err(|_| "SearXNG: compat Windows impossible".to_string())?;
    let stub = dir.join("pwd.py");
    std::fs::write(&stub, pwd_stub_body())
        .map_err(|_| "SearXNG: compat Windows impossible".to_string())?;
    Ok(Some(dir))
}

fn pwd_stub_body() -> &'static str {
    r#"from collections import namedtuple

struct_passwd = namedtuple("struct_passwd", "pw_name pw_passwd pw_uid pw_gid pw_gecos pw_dir pw_shell")

def getpwuid(uid):
    return struct_passwd("windows", "", uid, 0, "", "", "")
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwd_stub_exposes_getpwuid() {
        let body = pwd_stub_body();
        assert!(body.contains("def getpwuid"));
        assert!(body.contains("struct_passwd"));
    }
}
