/*extern crate iron;

use iron::prelude::*;
use iron::status;*/

//extern crate phant;
extern crate chrono;

use std::fs::File;
use std::io::{Read, Write};
//use std::time::SystemTime;
use std::env;


fn main() {
 /*   let mut phant = phant::Phant::new("http://data.sparkfun.com", 
                                      "pub"
                                      "priv", None);*/

    if env::args().count() != 3 {
        println!("Usage: {} <device> <logfile>", env::args().nth(0).unwrap());
        std::process::exit(1);
    }

    let device_path = env::args().nth(1).unwrap();
    let logfile_path = env::args().nth(2).unwrap();

    let mut tty_usb = File::open(device_path.clone()).ok().expect(format!("Could not open device {}", device_path).as_str());
    let mut log_file = File::create(logfile_path.clone()).ok().expect(format!("Could not open file {} to log to", logfile_path).as_str());

    let mut data: [u8; 512] = [0u8; 512];
    loop {
        let num_bytes = tty_usb.read(&mut data).unwrap();
        let line = std::str::from_utf8(&data[0..num_bytes]).ok().expect("Could not convert data from tty to UTF-8 string");
        let now = chrono::UTC::now();
        let data_str = format!("{}: {}", now.format("%b %-d, %-I:%M:%S%.3f").to_string(), line);
        println!("{}", data_str);
        log_file.write(data_str.into_bytes().as_slice());

//        phant.add("weight", line);
//        println!("Result of push: {}", phant.push().ok().expect("Pushing to server did not succeed"));
    }

    /*Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello World2!")))
    }).http("localhost:3000").unwrap();*/
}
