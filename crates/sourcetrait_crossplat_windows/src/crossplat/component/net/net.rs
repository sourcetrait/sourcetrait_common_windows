use crate::*;

pub struct WindowsNetComponentLookup;
impl cross::NetComponentLookup for WindowsNetComponentLookup {
    fn lookup_hostname(&self) -> cross::BridgeResult<String> {
        todo!()
        //winsys::lookup_hostname()
    }
    
    fn lookup_domain(&self) -> cross::BridgeResult<cross::Capable<Option<String>>> {
        cross::BridgeError::err_incapable(cross::Capability::Domains)
    }

    fn lookup_domain_authorities(&self) -> cross::BridgeResult<Vec<cross::DomainAuthority>> {
        cross::BridgeError::err_incapable(cross::Capability::Domains)
    }
}
