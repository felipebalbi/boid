#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use boid::run;
use uefi::prelude::*;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize services");

    let bt = system_table.boot_services();

    run(bt).expect("Simulation failure");

    Status::SUCCESS
}
