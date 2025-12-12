use crate::*;

const USER_INFO_23_LEVEL: u32 = 23;
const GROUP_USERS_INFO_0_LEVEL: u32 = 0;
const GROUP_INFO_3_LEVEL: u32 = 3;

pub(crate) struct UserInfo {
    pub(crate) username: PlatString,
    pub(crate) sid: String,
    pub(crate) domain: Option<PlatString>,
}

impl From<UserInfo> for User {
    fn from(value: UserInfo) -> Self {
        Self {
            ident: AccessIdent {
                name: value.username,
                domain: Capable::Capable(value.domain),
                id: Capable::Incapable,
                sid: Capable::Capable(value.sid),
            },
        }
    }
}

pub(crate) struct GroupInfo {
    pub(crate) name: PlatString,
    pub(crate) sid: String,
    pub(crate) domain: Option<PlatString>,
    
}

impl From<GroupInfo> for Group {
    fn from(value: GroupInfo) -> Self {
        Self {
            ident: AccessIdent {
                name: value.name,
                domain: Capable::Capable(value.domain),
                id: Capable::Incapable,
                sid: Capable::Capable(value.sid),
            },
        }
    }
}

/// looks up an sid string for an sid ptr owned by the provided
/// struct (like user_info_). frees bufptr on error.
///
/// SAFETY: bufptr will be freed on error. it's UB to use it after such.
pub(crate) fn win_lookup_struct_sid_string(sid_ptr: *mut c_void, bufptr: *mut u8) -> CrossResult<String> {
    let sid = unsafe {
        let mut string_sid: win::PWSTR = ptr::null_mut();
        
        let ret = win::ConvertSidToStringSidW(
            sid_ptr, // [in] Sid: PSID
            &mut string_sid, // [out] StringSid: LPWSTR*
        ); // [ret] BOOL

        if ret == 0 {
            win::NetApiBufferFree(bufptr as *mut _);
            return CrossError::err_not_found(ErrNoun::WindowsSID);
        }

        let sid_string = PlatString::from_wide_ptr(string_sid);
        win::LocalFree(string_sid as win::HLOCAL);

        match sid_string {
            PlatString::String(s) => s,
            _ => return CrossError::err_string(),
        }
    };

    Ok(sid)
}

pub(crate) fn win_lookup_qualified_user_info(domain: PlatStr<'_>, username: PlatStr<'_>) -> CrossResult<Option<UserInfo>> {
    win_lookup_any_user_info(username, Some(qualifying))
}
 
pub(crate) fn win_lookup_user_info(username: PlatStr<'_>) -> CrossResult<Option<UserInfo>>  {
    win_lookup_any_user_info(username, None)
}

pub(crate) fn win_lookup_any_user_info(username: PlatStr<'_>, qualifying: Option<&QualifyingAccessAuthority>) -> CrossResult<Option<UserInfo>>  {
    let mut bufptr: *mut u8 = ptr::null_mut();
    let server_name = match qualifying {
        Some(q) => q.authority().server_name().to_wide()?.as_ptr(),
        None => ptr::null(),
    };
    
    let ret = unsafe {
        win::NetUserGetInfo(
            server_name, // [in] servername: lpcwstr
            username.to_wide()?.as_ptr(), // [in] username: lpcwstr
            USER_INFO_23_LEVEL, // [in] level: dword
            &mut bufptr, // [out] butptr: LPBYTE*
        ) // [ret] NET_API_STATUS
    };

    match ret {
        win::NERR_Success => {},
        win::NERR_UserNotFound => return Ok(None),
        win::ERROR_ACCESS_DENIED => return CrossError::err_denied(ErrNoun::User),
        win::ERROR_BAD_NETPATH | win::NERR_InvalidComputer => return CrossError::err_not_found(ErrNoun::Domain),
        win::ERROR_INVALID_LEVEL => return CrossError::err_internal("Invalid user_info level"),
        _ => return CrossError::err_internal("Unexpected error kind"),
    }
    
    let user_info_23 = unsafe {
        let user_info = bufptr as *mut win::USER_INFO_23;
        *user_info
    };
    
    let username = PlatString::from_wide_ptr(user_info_23.usri23_name);
    dbg!(user_info_23.usri23_user_sid);
    let sid = win_lookup_struct_sid_string(user_info_23.usri23_user_sid, bufptr)?;

    let domain = qualifying.map(|q| q.domain().into());
    
    let user_info = UserInfo {
        username,
        sid,
        domain,
    };

    unsafe { win::NetApiBufferFree(bufptr as *mut _); }
    
    Ok(Some(user_info))
}

pub(crate) fn win_lookup_qualified_group_info(domain: PlatStr<'_>, groupname: PlatStr<'_>) -> CrossResult<Option<GroupInfo>> {
    let controller = win_lookup_domain_controller_info(domain)?
        .ok_or_else(|| CrossError::not_found(ErrNoun::AccessAuthority))?;
    
    win_lookup_authorative_group_info(domain, groupname, controller.server_name.as_plat_str())
}

pub(crate) fn win_lookup_authorative_group_info(domain: PlatStr<'_>, groupname: PlatStr<'_>, controller: PlatStr<'_>) -> CrossResult<Option<GroupInfo>>  {
    let mut buf_ptr: *mut u8 = ptr::null_mut();
    
    let ret = unsafe {
        win::NetGroupGetInfo(
            controller.to_wide()?.as_ptr(), // [in] servername: lpcwstr; null for local
            groupname.to_wide()?.as_ptr(), // [in] grupname: lpcwstr
            GROUP_INFO_3_LEVEL, // [in] level: dword
            &mut buf_ptr, // [out] bufptr: LPBYTE*
        ) // [ret] NET_API_STATUS
    };

    match ret {
        win::NERR_Success => {},
        win::NERR_GroupNotFound => return Ok(None),
        win::ERROR_ACCESS_DENIED => return CrossError::err_denied(ErrNoun::UserGroup),
        win::NERR_InvalidComputer => return CrossError::err_not_found(ErrNoun::Domain),
        win::ERROR_INVALID_LEVEL => return CrossError::err_internal("Invalid group_info level"),
        _ => return CrossError::err_internal("Unexpected error kind"),
    }
    
    let group_info_0 = unsafe {
        let group_info = buf_ptr as *mut win::GROUP_INFO_3;
        *group_info
    };

    let name = PlatString::from_wide_ptr(group_info_0.grpi3_name);
    let sid = win_lookup_struct_sid_string(group_info_0.grpi3_group_sid, buf_ptr)?;
    
    let group_info = GroupInfo {
        name, 
        sid,
        domain: Some(domain.into()),
    };
    
    unsafe { win::NetApiBufferFree(buf_ptr as *mut _); }
    
    Ok(Some(group_info))
}

pub(crate) fn win_lookup_authorative_user_groups_info(domain: PlatStr<'_>, username: PlatStr<'_>, controller: PlatStr<'_>) -> CrossResult<Vec<GroupInfo>>  {
    let mut buf_ptr: *mut u8 = ptr::null_mut();
    let mut entries_read: u32 = 0;
    let mut total_entries: u32 = 0;
    
    let ret = unsafe {
        win::NetUserGetGroups(
            controller.to_wide()?.as_ptr(), // [in] servername: LPCWSTR; null for local
            username.to_wide()?.as_ptr(), // [in] username: LPCWST
            GROUP_USERS_INFO_0_LEVEL, // [in] level: DWORD
            &mut buf_ptr, // [out] bufptr: LPBYTE*
            win::MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
        ) // [ret] NET_API_STATUS
    };

    match ret {
        win::NERR_Success => {},
        win::ERROR_ACCESS_DENIED => return CrossError::err_denied(ErrNoun::UserGroup),
        win::ERROR_INVALID_LEVEL => return CrossError::err_internal("Invalid group_info level"),
        win::NERR_UserNotFound => return CrossError::err_not_found(ErrNoun::User),
        //todo
        _ => return CrossError::err_internal("Unexpected error kind"),
    }
    
    let group_users_info_slice = unsafe {
        std::slice::from_raw_parts(
            buf_ptr as *const u8 as *const win::GROUP_USERS_INFO_0,
            entries_read as usize,
        )
    };

    let mut groupnames = Vec::with_capacity(entries_read as usize);
    for group_users_info_0 in group_users_info_slice {
        let groupname = PlatString::from_wide_ptr(group_users_info_0.grui0_name);
        groupnames.push(groupname);
    }

    unsafe { win::NetApiBufferFree(buf_ptr as *mut _); }

    let mut groups_info = Vec::with_capacity(groupnames.len()); 
    for groupname in groupnames {
        let group_info = win_lookup_authorative_group_info(domain, groupname.as_plat_str(), controller)?;
        if let Some(group_info) = group_info {
            groups_info.push(group_info);
        }
    }
    
    Ok(groups_info)
}

pub(crate) fn win_lookup_authorative_group_users_info(groupname: PlatStr<'_>, qualifying: &QualifyingAccessAuthority) -> CrossResult<Vec<UserInfo>>  {
    let mut buf_ptr: *mut u8 = ptr::null_mut();
    let mut entries_read: u32 = 0;
    let mut total_entries: u32 = 0;
    let resume_handle: usize = 0;
    
    let ret = unsafe {
        win::NetGroupGetUsers(
            qualifying.authority().server_name().to_wide()?.as_ptr(), // [in] servername: LPCWSTR; null for local
            groupname.to_wide()?.as_ptr(), // [in] groupname: LPCWST
            GROUP_USERS_INFO_0_LEVEL, // [in] level: DWORD
            &mut buf_ptr, // [out] bufptr: LPBYTE*
            win::MAX_PREFERRED_LENGTH, // [in] prefmaxlen: DWORD
            &mut entries_read, // [out] entriesread: LPDWORD
            &mut total_entries, // [out] totalentries: LPDWORD
            resume_handle as *mut usize, // [in, out] ResumeHandle: PDWORD_PTR
        ) // [ret] NET_API_STATUS
    };

    match ret {
        win::NERR_Success => {},
        win::ERROR_ACCESS_DENIED => return CrossError::err_denied(ErrNoun::UserGroup),
        win::ERROR_INVALID_LEVEL => return CrossError::err_internal("Invalid group_info level"),
        win::NERR_UserNotFound => return CrossError::err_not_found(ErrNoun::User),
        //todo
        _ => return CrossError::err_internal("Unexpected error kind"),
    }
    
    let group_users_info_slice = unsafe {
        std::slice::from_raw_parts(
            buf_ptr as *const u8 as *const win::GROUP_USERS_INFO_0,
            entries_read as usize,
        )
    };

    let mut usernames = Vec::with_capacity(entries_read as usize);
    for group_users_info_0 in group_users_info_slice {
        let username = PlatString::from_wide_ptr(group_users_info_0.grui0_name);
        usernames.push(username);
    }

    unsafe { win::NetApiBufferFree(buf_ptr as *mut _); }

    let mut users_info = Vec::with_capacity(usernames.len()); 
    for username in usernames {
        let user_info = win_lookup_authorative_user_info(domain, username.as_plat_str(), qualifying)?;
        if let Some(user_info) = user_info {
            users_info.push(user_info);
        }
    }
    
    Ok(users_info)
}

struct DomainControllerInfo {
    server_name: PlatString,
    domain: PlatString,
}

pub(crate) fn win_lookup_domain_controller_info(domain: PlatStr<'_>) -> CrossResult<Option<DomainControllerInfo>> {
    let info = unsafe {
        let mut domain_controller_info_ptr: *mut win::DOMAIN_CONTROLLER_INFOW = ptr::null_mut();
        
        let ret = win::DsGetDcNameW(
            ptr::null(),
            domain.to_wide()?.as_ptr(),
            ptr::null(),
            ptr::null(),
            win::DS_DIRECTORY_SERVICE_PREFERRED,
            &mut domain_controller_info_ptr
        );

        match ret {
            win::NERR_Success => {},
            _ => return Ok(None),
        }

        let controller_info = *domain_controller_info_ptr;
        let name = PlatString::from_wide_ptr(controller_info.DomainControllerName);

        win::NetApiBufferFree(domain_controller_info_ptr as *mut _);
        
        Some(DomainControllerInfo {
            server_name: name,
            domain: domain.into(),
        })
    };

    Ok(info)
}
