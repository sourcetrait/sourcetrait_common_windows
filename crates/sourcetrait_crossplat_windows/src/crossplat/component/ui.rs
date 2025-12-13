use crate::*;

pub struct WindowsUiComponentLookup;
impl cross::UiComponentLookup for WindowsUiComponentLookup {
    fn lookup_has_command_line(&self) -> cross::BridgeResult<bool> {
        const ENV_SESSION_NAME: &'static str = "SESSION_NAME";
        const VAL_CONSOLE: &'static str = "Console";
        
        match env::var(ENV_SESSION_NAME) {
            Ok(v) if v == VAL_CONSOLE => Ok(true),
            Ok(_) => Ok(false),
            Err(env::VarError::NotPresent) => Ok(false),
            Err(source) => cross::BridgeError::err_env_var(ENV_SESSION_NAME, source),
        }
    }

    fn lookup_has_graphical(&self) -> cross::BridgeResult<bool> {
        // winsys lookup:
        // 1. GetProcessWindowStation()
        // 2. GetUserObjectInformation(hWindowStation, UOI_IO, &is_interactive, sizeof(bool), null)
        todo!()
    }
}