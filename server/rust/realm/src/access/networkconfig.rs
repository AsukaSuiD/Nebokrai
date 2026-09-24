//! Network-конфигурация роли AuthServer, перенесённая в Realm `access/`.

pub struct AuthNetworkConfig {
    pub host_port: u32,
    pub max_login_servers: i32,
    pub max_in_flight_sends: i32,
    pub permitted_send_bytes: i32,
    pub new_accept_timeout_ms: i32,
}

impl AuthNetworkConfig {
    pub const fn new(
        host_port: u32,
        max_login_servers: i32,
        max_in_flight_sends: i32,
        permitted_send_bytes: i32,
        new_accept_timeout_ms: i32,
    ) -> Self {
        Self {
            host_port,
            max_login_servers,
            max_in_flight_sends,
            permitted_send_bytes,
            new_accept_timeout_ms,
        }
    }
}
