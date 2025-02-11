use sfc5xxx_rs::{device::Device, serialport};

fn main() {
    let test_port = serialport::new("COM4", 115200).open_native().unwrap();

    let mut device = Device::new(test_port, 0);
    let out = device.send_message();
    println!("{:?}", out);
}
