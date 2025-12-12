use crate::*;
use crate::windows::*;

impl TryFrom<PlatStr<'_>> for U16CString {
    type Error = CrossError;

    fn try_from(value: PlatStr<'_>) -> Result<Self, Self::Error> {
        value.to_wide()
    }
}

impl PlatString {
    pub fn from_wide_ptr(lp: *mut u16) -> Self {
        let cwstr = unsafe { U16CStr::from_ptr_str(lp) };
        match cwstr.to_string() {
            Ok(s) => Self::String(s),
            Err(_) => Self::OsString(cwstr.to_os_string()),
        }
    }
    
    pub fn to_wide(&self) -> CrossResult<U16CString> {
        U16CString::from_os_str(OsStr::new(self))
            .map_err(|_| CrossError::String)
    }
}

impl PlatStr<'_> {
    pub fn to_wide(&self) -> CrossResult<U16CString> {
        U16CString::from_os_str(OsStr::new(self))
            .map_err(|_| CrossError::String)
    }
}
