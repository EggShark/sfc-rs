// example taken from https://sensirion.github.io/python-uart-sfx6xxx/execute-measurements.html#example-script
use sfc6xxx_rs::device::{Device, DeviceError, StateResponseError};

fn main() {
    let port = serialport::new("/dev/ttyUSB0", 115200).open_native().unwrap();
    let mut device = Device::new(port, 0).unwrap();
    device.reset_device().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let serial_number = device.get_serial_number().unwrap();
    println!("serial number: {}", serial_number);
    device.set_setpoint(2.0).unwrap();

    for _ in 0..200 {
        let res = device.read_average_measured_value(50);
        match res {
            Ok(value) => println!("average_measured_value: {:?}", value),
            Err(DeviceError::StateResponse(StateResponseError::MeasureLoopNotRunning)) => {
                println!("Most likely the valve was closed due to overheating protection.\nMake sure a flow is applied and start the script again");
                break;
            }
            _ => {res.unwrap();},
        }
    }
    device.set_setpoint(0.0).unwrap();
}
