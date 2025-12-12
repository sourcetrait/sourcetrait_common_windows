
pub(crate) mod win {
    pub(crate) use windows_sys::{
        core::{PCWSTR, PWSTR, HSTRING},
        Win32::{
            Foundation::{
                GetLastError, ERROR_INVALID_SID,
                LocalFree, HLOCAL,
                ERROR_ACCESS_DENIED, ERROR_BAD_NETPATH, ERROR_INVALID_LEVEL,
                ERROR_INVALID_PARAMETER,
            },
            Networking::{
                ActiveDirectory::{
                    DsGetDcNameW, DS_DIRECTORY_SERVICE_PREFERRED,
                    DOMAIN_CONTROLLER_INFOW,
                }
            },
            NetworkManagement::NetManagement::{
                NetUserGetInfo, NetGroupGetInfo, NetUserGetGroups,
                NetGroupGetUsers, NetApiBufferFree,
                MAX_PREFERRED_LENGTH, USER_INFO_23, GROUP_USERS_INFO_0, GROUP_INFO_3,
                NERR_Success, NERR_UserNotFound, NERR_GroupNotFound,
                NERR_InvalidComputer,
            },
            Security::{
                Authorization::{
                    ConvertSidToStringSidW,
                },
                PSID,
            },
        },
    };
}
