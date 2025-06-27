# SFC6xxx-rs
A pure rust implementation of the SHDLC driver for Sensirions SFC6xxx mass flow controllers. The api was made to model the [official python library](https://sensirion.github.io/python-uart-sfx6xxx/), while adding rust best practices. The bare minimum code needed to get started looks like:
```rust
let port = serialport::new("/dev/ttyUSB0", 115200).open_native().unwrap();
let mut device = Device::new(port, 0).unwrap();
// set the devices flow rate
device.set_setpoint(4).unwrap();
// read in the measured value of the device
device.read_measured_value();

```

### Testing
All device functions have an associated test that were passing on a SFC6000D-5slm
