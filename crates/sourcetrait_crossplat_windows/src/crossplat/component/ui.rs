use crate::*;

pub struct LinuxUiComponentLookup;
impl cross::UiComponentLookup for LinuxUiComponentLookup {
    fn lookup_has_command_line(&self) -> cross::BridgeResult<bool> {
        match env::var(unix::ENV_TERM) {
            Ok(_) => Ok(true),
            Err(env::VarError::NotPresent) => Ok(false),
            Err(source) => cross::BridgeError::err_env_var(unix::ENV_VAR_TERM, source),
        }
    }

    fn lookup_has_graphical(&self) -> cross::BridgeResult<bool> {
        match env::var(ENV_WAYLAND_DISPLAY) {
            Ok(_) => Ok(true),
            Err(env::VarError::NotPresent) => Ok(false),
            Err(source) => cross::BridgeError::err_env_var(ENV_WAYLAND_DISPLAY, source),
        }
    }
}