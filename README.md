# SFC-RS
A set of libraries to interface with Sensirion's mass flow controllers.

## sfc-core
Sfc-core is the core library contianing the shared types used across device types. It also contains the code to decode and encode data streams to be sent and recivied from the device.

## sfc6xxx-rs
This module is the pure rust implementation of the SHDLC driver for the SFC6xxx devices. Its fully functional and up and running

## sfc5xxx-rs
This module is the pure rust implementation of the SHDLC driver for the SFC5xxx devices. All commands have been implmented but are untested and need validation.
