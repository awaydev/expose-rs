use std::{ io::{ Read, Write }, net::TcpStream, str::Lines };
use std::net::SocketAddr;

pub struct Url {
    pub protocol: String,
    pub host:     String,
    pub path:     String
}

pub fn find_line<'a> (name: &str, lines: Lines<'a>) -> Option<&'a str> {
    let name = name.to_uppercase();
    let name = &format!("{name}: ");
    for line in lines {
        if line.starts_with(name) {
            return Some(line.trim_start_matches(name).trim())
        }
    }
    None
}

#[macro_export]
macro_rules! find_lines {
    ($list:expr, $($l:expr),+) => {
        ($(
            find_line($l, $list),
        )+)
    };
}

pub fn get_xml_tag_childs (xml: &str, tag: &str) -> Option<String> {
    let open_tag = &format!("<{tag}>");
    let close_tag = &format!("</{}>", tag.split_whitespace().nth(0)?);

    if let Some(x) = xml.find(open_tag) {
        let start = x + open_tag.len();
        if let Some(y) = &xml[start..].find(close_tag) {
            let end = start + y;
            return Some(xml[start .. end].into())
        }
    }

    None
}

pub fn format_url (url: &str) -> Url {
    let url = url.replace("://", " ").replacen("/", " ", 1);
    let mut url = url.split_whitespace();
    Url {
        protocol: url.next().expect("URL to format is not correct: protocol is missing").into(),
        host: url.next().expect("URL to format is not correct: host is missing").into(),
        path: url.next().unwrap_or("").into()
    }
}

pub fn open_http (url: &Url, method: &str, other: &str) -> Result<String, std::io::Error> {
    let (host, path) = (&url.host, &url.path);

    let method = method.to_uppercase();

    let mut stream = TcpStream::connect(host)?;
    let request = format!("{method} /{path} HTTP/1.1\r\n\
    HOST: {host}\r\n\
    {other}
    \r\n");

    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

pub fn get_bind () -> Result<SocketAddr, std::io::Error> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;

    socket.local_addr()
}