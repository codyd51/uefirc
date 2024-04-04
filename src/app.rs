use alloc::format;
use alloc::rc::Rc;
use log::info;
use uefi::prelude::BootServices;
use uefi::table::boot::ScopedProtocol;
use crate::connection::{get_tcp_protocol, TcpConnection};
use crate::connection::get_tcp_service_binding_protocol;
use crate::ipv4::IPv4Address;
use crate::tcpv4::TCPv4ServiceBindingProtocol;

#[derive(Debug)]
pub struct IrcClient<'a> {
    boot_services: &'static BootServices,
    tcp_service_binding_protocol: ScopedProtocol<'a, TCPv4ServiceBindingProtocol>,

    pub active_connection: Option<Rc<TcpConnection<'a>>>,
}

impl<'a> IrcClient<'a> {
    pub fn new(
        boot_services: &'static BootServices,
    ) -> Self {
        let tcp_service_binding_protocol = get_tcp_service_binding_protocol(boot_services);

        Self {
            boot_services,
            tcp_service_binding_protocol,
            active_connection: None,
        }
    }

    pub fn connect_to_server_and_register(
        &mut self,
        ip_address: IPv4Address,
        port: u16,
        nickname: &str,
        real_name: &str,
    ) {
        info!("Initializing connection to IRC server...");
        let tcp_protocol = get_tcp_protocol(
            self.boot_services,
            &self.tcp_service_binding_protocol,
        );

        let connection = TcpConnection::new(
            self.boot_services,
            tcp_protocol,
            ip_address,
            port,
        );
        self.active_connection = Some(connection);
        self.set_nickname(nickname);
        self.set_user(nickname, real_name);
    }

    pub fn send_line_command(&mut self, command: &str) {
        let data = format!("{command}\r\n").into_bytes();
        let conn = self.active_connection.as_mut();
        conn.unwrap().transmit(&data);
    }

    pub fn set_nickname(&mut self, nickname: &str) {
        self.send_line_command(&format!("NICK {nickname}"))
    }

    pub fn send_message_to_user(&mut self, user: &str, message: &str) {
        self.send_line_command(&format!("PRIVMSG {user} :{message}"))
    }

    pub fn send_message_to_channel(&mut self, channel: &str, message: &str) {
        // TODO(PT): Auto-join the channel if not already joined?
        self.send_line_command(&format!("PRIVMSG #{channel} :{message}"))
    }

    pub fn join_channel(&mut self, channel: &str) {
        // TODO(PT): Block if we've already joined this channel?
        self.send_line_command(&format!("JOIN #{channel}"))
    }

    pub fn set_user(&mut self, nickname: &str, real_name: &str) {
        self.send_line_command(&format!("USER {nickname} 0 * :{real_name}"))
    }

    // TODO(PT): Add an 'info bar' on the right that shows available channels/users
    // The primary cost is drawing, so we can only draw the first N channels
}
