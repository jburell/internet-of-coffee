/*extern crate iron;

use iron::prelude::*;
use iron::status;*/

extern crate libusb;

use libusb::Context;

fn main() {
	let ctx = Context::new().unwrap();
	let mut dymo = ctx.devices().unwrap().iter().filter(|dev| {
		let desc = dev.device_descriptor().unwrap();
		//println!("Filter {:x} == 0x922, {:x} == 0x8003", desc.vendor_id(), desc.product_id());
		0x922 == desc.vendor_id() && 0x8003 == desc.product_id()
 	}).collect::<Vec<libusb::Device>>().pop().unwrap();

	println!("Vendor: {}", dymo.device_descriptor().unwrap().vendor_id());

	let mut handle = match dymo.open() {
		Ok(handle) => Some(handle),
		Err(e) => {println!("Error: {}", e); None},
	};

	match handle {
		Some(mut handle) => {handle.reset(); println!("Active conf: {}", handle.active_configuration().unwrap()); ()},
		None => println!("Could not reset the handle"),
 	}
    /*Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello World2!")))
    }).http("localhost:3000").unwrap();*/
}
