use log::info;
use uefi::prelude::BootServices;
use uefi::table::boot::ScopedProtocol;
use crate::connection::{get_tcp_protocol, TcpConnection};
use crate::connection::get_tcp_service_binding_protocol;
use crate::ipv4::IPv4Address;
use crate::tcpv4::TCPv4ServiceBindingProtocol;

#[derive(Debug)]
pub struct IrcClient<'a> {
    boot_services: &'a BootServices,
    tcp_service_binding_protocol: ScopedProtocol<'a, TCPv4ServiceBindingProtocol>,

    pub active_connection: Option<TcpConnection<'a>>,
}

impl<'a> IrcClient<'a> {
    pub fn new(
        boot_services: &'a BootServices,
    ) -> Self {
        let tcp_service_binding_protocol = get_tcp_service_binding_protocol(boot_services);

        Self {
            boot_services,
            tcp_service_binding_protocol,
            active_connection: None,
        }
    }

    fn connect_to_server(&mut self) {
        info!("Initializing connection to IRC server...");
        let tcp_protocol = get_tcp_protocol(
            self.boot_services,
            &self.tcp_service_binding_protocol,
        );

        let connection = TcpConnection::new(
            self.boot_services,
            tcp_protocol,
            IPv4Address::new(93, 158, 237, 2),
            6665,
        );
        self.active_connection = Some(connection);
    }

    pub fn step(&mut self) {
        if self.active_connection.is_none() {
            self.connect_to_server();
        }
        let mut connection = self.active_connection.as_mut().expect("We should always be connected to the server now.");
        connection.step();
    }
}
