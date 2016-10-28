/*extern crate iron;

use iron::prelude::*;
use iron::status;*/

//extern crate phant;
extern crate chrono;
extern crate regex;

use std::fs::File;
use std::io::{Read, Write};
//use std::time::SystemTime;
use std::env;
use regex::Regex;


fn main() {
    /*   let mut phant = phant::Phant::new("http://data.sparkfun.com",
                                         "pub"
                                         "priv", None);*/

    if env::args().count() != 5 {
        println!("Usage: {} <device> <logfile> <lower_limit_in_grams> <upper_limit_in_grams>",
                 env::args().nth(0).unwrap());
        std::process::exit(1);
    }

    let device_path = env::args().nth(1).unwrap();
    let logfile_path = env::args().nth(2).unwrap();

    let lower_limit = env::args().nth(3).unwrap().parse::<u32>().unwrap();
    let upper_limit = env::args().nth(4).unwrap().parse::<u32>().unwrap();

    let mut tty_usb = File::open(device_path.clone())
        .ok().expect(format!("Could not open device {}", device_path).as_str());
    let mut log_file = File::create(logfile_path.clone())
        .ok().expect(format!("Could not open file {} to log to", logfile_path).as_str());

    let regex_pattern = r"\d+";
    let weight_matcher = Regex::new(regex_pattern).unwrap();

    let mut data: [u8; 512] = [0u8; 512];
    loop {
        let num_bytes = tty_usb.read(&mut data).unwrap();
        let line = std::str::from_utf8(&data[0..num_bytes])
            .ok().expect("Could not convert data from tty to UTF-8 string").trim();

        let now = chrono::UTC::now();
        let data_str = format!("{}: {}", now.format("%b %-d, %-I:%M:%S%.3f").to_string(), line);
        let caps = weight_matcher.captures(line);
        let status_str = match caps {
            Some(c) => c.at(0).unwrap(),
            None => "",
        };
        let coffee_status = match status_str.parse::<u32>() {
            Ok(r) =>
                match r {
                    r if r < lower_limit => "LOW",
                    r if r > upper_limit => "HIGH",
                    _ => "NORMAL",
                },
            Err(_) => "UNKNOWN",
        };
        println!("{}Coffee level: {}", data_str, coffee_status);
        let _ = log_file.write(data_str.into_bytes().as_slice());

        //        phant.add("weight", line);
        //        println!("Result of push: {}", phant.push().ok().expect("Pushing to server did not succeed"));
    }

    /*Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello World2!")))
    }).http("localhost:3000").unwrap();*/
}
