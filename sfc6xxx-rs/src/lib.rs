//! # SFC6XXX-rs
//! This libraray is meant to provide an platform independant rust driver for
//! Sensirion's SFC6xxx mass flow controllers. The code was based arround the official
//! [python SHDLC](https://sensirion.github.io/python-uart-sfx6xxx/) library and should match
//! match its interface, while using Rust's powerful Result type.
//! ## Testing
//! Several tests have been written to tests this library's functionality and a majority of them
//! are in device.rs. Most test reads and checks values but several functions like
//! [get_serial_number](device::Device::get_serial_number) and [get_article_code](device::Device::get_article_code)
//! cannot be accuratley tested. In these cases the code checks to see if the response errored and nothing else.

pub mod device;
pub use serialport;
