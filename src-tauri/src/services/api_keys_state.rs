use std::sync::Mutex;

struct VaultState {
    master_key: Zeroizing<Vec<u8>>,
    keys: HashMap<String, Zeroizing<String>>,
}

static STATE: std::sync::LazyLock<Mutex<Option<VaultState>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

struct ZeroizingMap(HashMap<String, String>);

impl Drop for ZeroizingMap {
    fn drop(&mut self) {
        for value in self.0.values_mut() {
            value.zeroize();
        }
    }
}
