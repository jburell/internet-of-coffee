/*extern crate iron;

use iron::prelude::*;
use iron::status;*/
extern crate sdl2;
extern crate sdl2_ttf;

//extern crate phant;
extern crate chrono;
extern crate regex;
extern crate ansi_term;

use std::env;
use std::path::Path;

use std::fs::File;
use std::io::{Read, Write};
//use std::time::SystemTime;
use regex::Regex;
use ansi_term::Colour;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::render::Texture;

use sdl2_ttf::Font;

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 600;

struct LevelTextures {
    high: Texture,
    normal: Texture,
    low: Texture,
    texture_label: Texture,

    target_level: Rect,
    target_label: Rect,
}

enum CoffeeLevel {
    HIGH,
    NORMAL,
    LOW,
}

struct LevelConfig {
    max: u32,
    min: u32,
}


fn main() {
    /*   let mut phant = phant::Phant::new("http://data.sparkfun.com",
                                         "pub"
                                         "priv", None);*/

    if env::args().count() != 5 {
        println!("Usage: {} <device> <logfile> <lower_limit_in_grams> <upper_limit_in_grams>",
                 //println!("Usage: {} <lower_limit_in_grams> <upper_limit_in_grams>",
                 env::args().nth(0).unwrap());
        std::process::exit(1);
    }

    let device_path = env::args().nth(1).unwrap();
    let logfile_path = format!("{}{}",
                               env::args().nth(2).unwrap(),
                               chrono::UTC::now().format("%F@%H-%M-%S"));


    let mut tty_usb = File::open(device_path.clone())
        .ok().expect(format!("Could not open device {}", device_path).as_str());
    let mut log_file = File::create(logfile_path.clone())
        .ok().expect(format!("Could not open file {} to log to", logfile_path).as_str());


    let lower_limit = env::args().nth(3).unwrap().parse::<u32>().unwrap();
    let upper_limit = env::args().nth(4).unwrap().parse::<u32>().unwrap();

    main_loop(&mut tty_usb, &mut log_file, lower_limit, upper_limit);

    /*Iron::new(|_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello World2!")))
    }).http("localhost:3000").unwrap();*/
}

fn main_loop(tty_usb: &mut File, log_file: &mut File, lower_limit: u32, upper_limit: u32) {
    let path: &Path = Path::new("fonts/comicbd.ttf");
    run(path, &LevelConfig { max: upper_limit, min: lower_limit }, tty_usb, log_file);

    /*loop {

        //        phant.add("weight", line);
        //        println!("Result of push: {}", phant.push().ok().expect("Pushing to server did not succeed"));
    }*/
}

fn read_and_log(tty_usb: &mut File, mut log_file: &mut File, level_config: &LevelConfig) -> Option<u32> {
    let mut data: [u8; 512] = [0u8; 512];
    let num_bytes = tty_usb.read(&mut data).unwrap();
    match std::str::from_utf8(&data[0..num_bytes]) {
        Ok(l) => Some(handle_value(l.trim(), level_config, &mut log_file)),
        Err(e) => {
            // "Could not convert data from tty to UTF-8 string"
            println!("{}", Colour::Purple.paint(e.to_string()));
            None
        },
    }
}

fn handle_value(line: &str, level_config: &LevelConfig, log_file: &mut File) -> u32 {
    if line.len() == 0 { 0 } else {
        let regex_pattern = r"\d+";
        let weight_matcher = Regex::new(regex_pattern).unwrap();
        let now = chrono::UTC::now();
        let data_str = format!("{}: {}", now.format("%b %-d, %-I:%M:%S%.3f").to_string(), line);
        let caps = weight_matcher.captures(line);
        let status_str = match caps {
            Some(c) => c.at(0).unwrap(),
            None => "",
        };

        let parse_res = match status_str.parse::<u32>() {
            Ok(r) =>
                (match select_level(r, level_config) {
                    CoffeeLevel::HIGH => Colour::Green.paint("HIGH"),
                    CoffeeLevel::NORMAL => Colour::Yellow.paint("NORMAL"),
                    CoffeeLevel::LOW => Colour::Red.paint("LOW"),
                }, Some(r)),
            Err(_) => (Colour::Cyan.paint("UNKNOWN"), None),
        };

        println!("{}\nCoffee level: {}", data_str, parse_res.0);

        if parse_res.1 != None {
            let _ = log_file.write(data_str.into_bytes().as_slice());
            parse_res.1.unwrap()
        } else {
            0
        }
    }
}

macro_rules! rect (
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        println!("Scaling down! The text will look worse!");
        if wr > hr {
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (SCREEN_WIDTH as i32 - w) / 2;
    let cy = (SCREEN_HEIGHT as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

fn init_gfx(font: &mut Font, renderer: &mut Renderer) -> LevelTextures {
    let surface_label = font.render("Level:")
        .blended(Color::RGBA(196, 151, 102, 255)).unwrap();
    let surface_level_high = font.render("HIGH")
        .blended(Color::RGBA(0, 207, 20, 255)).unwrap();
    let surface_level_normal = font.render("NORMAL")
        .blended(Color::RGBA(255, 194, 51, 255)).unwrap();
    let surface_level_low = font.render("LOW")
        .blended(Color::RGBA(255, 43, 26, 255)).unwrap();

    let mut texture_label = renderer.create_texture_from_surface(&surface_label).unwrap();
    let mut texture_level_high = renderer.create_texture_from_surface(&surface_level_high).unwrap();
    let mut texture_level_normal = renderer.create_texture_from_surface(&surface_level_normal).unwrap();
    let mut texture_level_low = renderer.create_texture_from_surface(&surface_level_low).unwrap();

    let TextureQuery { width, height, .. } = texture_label.query();
    let mut width_label = width;
    let mut height_label = height;
    let TextureQuery { width, height, .. } = texture_level_normal.query();
    let mut width_level = width;
    let mut height_level = height;

    // If the example text is too big for the screen, downscale it (and center irregardless)
    let padding = 32;
    let mut target_label = get_centered_rect(width_label, height_label, SCREEN_WIDTH - padding, SCREEN_HEIGHT - padding);
    let mut target_level = get_centered_rect(width_level, height_level, SCREEN_WIDTH - padding, SCREEN_HEIGHT - padding);

    let new_y_label = (target_label.y() as f32 - (target_label.height() as f32 / 2f32)) as i32;
    let new_y_level = (target_level.y() as f32 + (target_level.height() as f32 / 2f32)) as i32;
    target_label.set_y(new_y_label);
    target_level.set_y(new_y_level);

    LevelTextures {
        high: texture_level_high,
        normal: texture_level_normal,
        low: texture_level_low,
        texture_label: texture_label,

        target_label: target_label,
        target_level: target_level,
    }
}


fn select_level(weight: u32, config: &LevelConfig) -> CoffeeLevel {
    let padding = ((config.max - config.min) as f32 * 0.2f32) as u32;
    match weight {
        w if w > config.max - padding => CoffeeLevel::HIGH,
        w if w < config.min + padding => CoffeeLevel::LOW,
        _ => CoffeeLevel::NORMAL,
    }
}

fn select_tex_for_level(level: CoffeeLevel, tex_levels: &LevelTextures) -> &Texture {
    match level {
        CoffeeLevel::HIGH => &tex_levels.high,
        CoffeeLevel::NORMAL => &tex_levels.normal,
        CoffeeLevel::LOW => &tex_levels.low,
    }
}

fn run(font_path: &Path, level_config: &LevelConfig, tty_usb: &mut File, mut log_file: &mut File) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();


    let window = video_subsys.window("SDL2_TTF Example", SCREEN_WIDTH, SCREEN_HEIGHT)
    .position_centered()
    .opengl()
    .build()
    .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    renderer.set_draw_color(Color::RGBA(102, 58, 23, 255)); // Brown
    renderer.clear();
    renderer.present();

    // Load a font
    let mut font = ttf_context.load_font(font_path, 128).unwrap();
    let mut font_percent = ttf_context.load_font(font_path, 32).unwrap();
    font.set_style(sdl2_ttf::STYLE_BOLD);
    font_percent.set_style(sdl2_ttf::STYLE_BOLD);

    let mut tex_levels = init_gfx(&mut font, &mut renderer);

    'mainloop: loop {
        // factor this into a separate thread/future/concept for concurrency of the day
        match read_and_log(tty_usb, &mut log_file, level_config) {
            Some(weight) => {
                renderer.set_draw_color(Color::RGBA(102, 58, 23, 255)); // Brown
                renderer.clear();

                let mut tex_level = select_tex_for_level(select_level(weight, level_config), &tex_levels);
                renderer.copy(&tex_levels.texture_label, None, Some(tex_levels.target_label));
                renderer.copy(&mut tex_level, None, Some(tex_levels.target_level));


                let corrected_weight = if weight < level_config.min { level_config.min } else { weight };
                let mut coffee_ratio = (corrected_weight - level_config.min) as f32 / (level_config.max - level_config.min) as f32;
                let coffee_percent = if coffee_ratio < 0f32 { 0f32 } else { coffee_ratio * 100f32 };
                let surface_coffee_percent = font_percent.render(format!("{}% kaffe", coffee_percent).as_str())
                .blended(Color::RGBA(255, 255, 255, 255)).unwrap();
                let mut coffee_tex = renderer.create_texture_from_surface(&surface_coffee_percent).unwrap();
                let TextureQuery { width, height, .. } = coffee_tex.query();
                let coffe_tex_rect = rect!(SCREEN_WIDTH - 32 - width, SCREEN_HEIGHT - 32 - height, width, height);
                renderer.copy(&coffee_tex, None, Some(coffe_tex_rect));

                renderer.present();
            },
            None => {},
        }

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {}
            }
        }
    }
}