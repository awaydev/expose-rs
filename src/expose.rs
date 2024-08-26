use core::str;
use std::{ net::{ Ipv4Addr, UdpSocket }, time::Duration };

use crate::{ find_lines, utils::{ format_url, get_bind, find_line, get_xml_tag_childs, open_http, Url } };

macro_rules! soap {
    ($($body:expr),+) => {
        "<?xml version=\"1.0\"?>\r\n\
        <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\r\n\
        <s:Body>\r\n".to_owned() + $($body + "\r\n" +)+ "</s:Body>\r\n\
        </s:Envelope>\r\n"
    };
}

macro_rules! component {
    ($name:expr, $content:expr) => {
        &format!("<u:{} xmlns:u=\"urn:schemas-upnp-org:service:WANIPConnection:1\">{}</u:{}>", $name, $content, $name)
    };
}

pub struct Session {
    pub device_addr: String,
    pub name:        String,
    pub local_ip:    String,
    pub endpoint:    String,
}

const MULTICASTADDR: &str = "239.255.255.250:1900";

pub fn discover () -> Result<Session, std::io::Error> {
    let discovery_message = format!("M-SEARCH * HTTP/1.1\r\n\
    HOST: {}\r\n\
    MAN: \"ssdp:discover\"\r\n\
    MX: 2\r\n\
    ST: urn:schemas-upnp-org:service:WANIPConnection:1\r\n\
    \r\n", MULTICASTADDR);

    let bind = get_bind()?;
    let socket = UdpSocket::bind(bind).expect("Exposer failed to bind socket");
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;

    socket.send_to(discovery_message.as_bytes(), MULTICASTADDR)?;

    let mut buf = [0; 2048];
    let amt = socket.recv(&mut buf)?;

    let response = str::from_utf8(&buf[..amt]).unwrap();

    let (location, server) = find_lines!(response.lines(), "location", "server");

    match (location, server) {
        (Some(location), Some(server)) => {
            let url = format_url(location);

            let igd = open_http(&url, "GET", "")?;
            if let Some(x) = igd.find("urn:upnp-org:serviceId:WANIPConn1") {
                let endpoint = get_xml_tag_childs(&igd[x..], "controlURL");
                if let Some(endpoint) = endpoint {
                    return Ok(Session {
                        endpoint: endpoint.replacen("/", "", 1),
                        device_addr: url.host,
                        name: server.split_whitespace().last().unwrap().to_string(),
                        local_ip: bind.ip().to_string()
                    })
                }
            }

            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found endpoint for WANIPConnection1"))
        },
        _ => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found location and name of device"))
    }
}

pub fn get_external_ip (session: &Session) -> Result<Ipv4Addr, std::io::Error> {
    let soap_request = soap!(
        component!("GetExternalIPAddress", "")
    );

    let response = open_http(&Url {
        protocol: "http".to_string(),
        host: session.device_addr.clone(),
        path: session.endpoint.clone()
    }, "POST",
    &format!("CONTENT-TYPE: text/xml; charset=\"utf-8\"\r\n\
    SOAPACTION: \"urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress\"\r\n\
    CONTENT-LENGTH: {}\r\n\
    \r\n\
    {soap_request}", soap_request.len()))?;

    if let Some(ip) = get_xml_tag_childs(&response, "NewExternalIPAddress") {
        return Ok(ip.parse::<Ipv4Addr>().unwrap())
    }
    else {
        Err(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "Cannot get external IP address"))
    }
}

pub fn forward_port (session: &Session, protocol: &str, internal_port: u16, external_port: u16, description: &str, duration: usize) -> Result<(), std::io::Error> {
    let local_ip = &session.local_ip;
    let soap_request = soap!(
        component!("AddPortMapping", format!("<NewRemoteHost></NewRemoteHost>\r\n\
        <NewExternalPort>{external_port}</NewExternalPort>\r\n\
        <NewProtocol>{protocol}</NewProtocol>\r\n\
        <NewInternalPort>{internal_port}</NewInternalPort>\r\n\
        <NewInternalClient>{local_ip}</NewInternalClient>
        <NewEnabled>1</NewEnabled>\r\n\
        <NewPortMappingDescription>{description}</NewPortMappingDescription>\r\n\
        <NewLeaseDuration>{duration}</NewLeaseDuration>
        "))
    );

    open_http(&Url {
        protocol: "http".to_string(),
        host: session.device_addr.clone(),
        path: session.endpoint.clone()
    }, "POST", &format!("CONTENT-TYPE: text/xml; charset=\"utf-8\"\r\n\
    SOAPACTION: \"urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping\"\r\n\
    CONTENT-LENGTH: {}\r\n\
    \r\n\
    {soap_request}", soap_request.len()))?;
    
    Ok(())
}

pub fn remove_port (session: &Session, protocol: &str, external_port: u16) -> Result<(), std::io::Error> {
    let soap_request = soap!(
        component!("DeletePortMapping", format!("<NewRemoteHost></NewRemoteHost>\r\n\
        <NewExternalPort>{external_port}</NewExternalPort>\r\n\
        <NewProtocol>{protocol}</NewProtocol>\r\n\
        "))
    );
    
    open_http(&Url {
        protocol: "http".to_string(),
        host: session.device_addr.clone(),
        path: session.endpoint.clone()
    }, "POST", &format!("CONTENT-TYPE: text/xml; charset=\"utf-8\"\r\n\
    SOAPACTION: \"urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping\"\r\n\
    CONTENT-LENGTH: {}\r\n\
    \r\n\
    {soap_request}", soap_request.len()))?;

    Ok(())
}