use std::net::UdpSocket;

const RESP_BUF: [u8; 9] = [
    'j' as u8,
    's' as u8,
    'e' as u8,
    'm' as u8,
    ' ' as u8,
    't' as u8,
    'a' as u8,
    'd' as u8,
    'y' as u8,
];

pub fn reply_to_broadcasts() {
    println!("Starting replier");

    let socket: UdpSocket = loop {
        if let Ok(socket) = UdpSocket::bind(("0.0.0.0", crate::REPLY_PORT)) {
            break socket;
        }
    };
    println!("Replier started");

    loop {
        let mut buf = [0u8; 8092];
        if let Ok((_size, address)) = socket.recv_from(&mut buf) {
            let data = buf.into_iter().map(|u| u as char).collect::<String>();
            println!("Received broadcast\nAddress: {}\nBuf: {:?}", address, data.trim_end_matches("\0"));
            let _ = socket.send_to(&RESP_BUF, address);
        }
    }
}
