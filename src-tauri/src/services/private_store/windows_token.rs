use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_INSUFFICIENT_BUFFER, HANDLE,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, IsValidSid, TokenUser, PSID, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

const MAX_TOKEN_INFO: usize = 65_536;
const ERROR: &str = "stockage privé indisponible";

pub struct CurrentUser {
    _storage: Vec<usize>,
    sid: PSID,
}

impl CurrentUser {
    pub fn sid(&self) -> PSID {
        self.sid
    }
}

pub fn current_user() -> Result<CurrentUser, String> {
    let token = open_process_token()?;
    let mut required = 0_u32;
    let first =
        unsafe { GetTokenInformation(token.0, TokenUser, std::ptr::null_mut(), 0, &mut required) };
    if first != 0 || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER {
        return Err(ERROR.to_string());
    }

    let size = usize::try_from(required).map_err(|_| ERROR.to_string())?;
    if !(std::mem::size_of::<TOKEN_USER>()..=MAX_TOKEN_INFO).contains(&size) {
        return Err(ERROR.to_string());
    }
    let words = size.div_ceil(std::mem::size_of::<usize>());
    let mut storage = vec![0_usize; words];
    let success = unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            storage.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    };
    if success == 0 {
        return Err(ERROR.to_string());
    }

    let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
    if user.User.Sid.is_null() || unsafe { IsValidSid(user.User.Sid) } == 0 {
        return Err(ERROR.to_string());
    }
    Ok(CurrentUser {
        sid: user.User.Sid,
        _storage: storage,
    })
}

struct Token(HANDLE);

impl Drop for Token {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn open_process_token() -> Result<Token, String> {
    let mut token = std::ptr::null_mut();
    let success = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
    if success == 0 || token.is_null() {
        Err(ERROR.to_string())
    } else {
        Ok(Token(token))
    }
}
