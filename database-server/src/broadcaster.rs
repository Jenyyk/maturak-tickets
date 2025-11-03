#[link(name = "brdc_bacon", kind = "dylib")]
unsafe extern "C" {
    fn brdc_startBrdcast(port: u16, msg: *const std::ffi::c_char, msg_len: usize) -> u8;

    fn brdc_stopBrdcast();

    fn brdc_getError() -> *const std::ffi::c_char;
}

#[allow(dead_code)]
struct SocketHolder(u8);

impl Drop for SocketHolder {
    fn drop(&mut self) {
        unsafe {
            brdc_stopBrdcast();
        }
    }
}

pub fn start_broadcast() {
    let msg = std::ffi::CString::new(crate::DATAB_PORT.to_string()).unwrap();
    let msg_len = msg.as_bytes().len();

    println!("Starting broadcast");

    #[allow(unused_variables)]
    let holder = unsafe {
        let stat = brdc_startBrdcast(crate::BROAD_PORT, msg.as_ptr(), msg_len);
        if stat != 0 {
            let mut error = brdc_getError();

            print!("Error: ");
            while *error != 0 {
                print!("{}", *error as u8 as char);
                error = error.add(1);
            }
            println!();
        }

        SocketHolder(stat)
    };
    println!("Broadcast started");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
