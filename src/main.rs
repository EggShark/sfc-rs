use sfc5xxx_rs::{device::Device, serialport};

#[cfg(target_os="linux")]
const PORT: &str = "/dev/ttyUSB0";
#[cfg(target_os="windows")]
const PORT: &str = "COM4";

fn main() {
    let test_port = serialport::new(PORT, 115200).open_native().unwrap();

    let mut device = Device::new(test_port, 0);
    let out = device.get_serial_number();
    let oo = device.get_article_code();
    println!("{:?}", out);
    println!("{:?}", oo);
}
