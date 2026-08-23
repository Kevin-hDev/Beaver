use super::windows_snapshot::{AdapterCapacity, AdapterLuid};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND,
};

const MAX_DXGI_ADAPTERS: u32 = 64;

pub(super) fn capacities() -> Option<Vec<AdapterCapacity>> {
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1().ok()? };
    let mut adapters = Vec::new();
    for index in 0..MAX_DXGI_ADAPTERS {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(_) => return None,
        };
        let description = unsafe { adapter.GetDesc1().ok()? };
        adapters.push(AdapterCapacity {
            luid: AdapterLuid::new(
                description.AdapterLuid.HighPart,
                description.AdapterLuid.LowPart,
            ),
            total_bytes: description.DedicatedVideoMemory as u64,
            software: description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0,
        });
    }
    (!adapters.is_empty()).then_some(adapters)
}
