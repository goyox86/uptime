//#![deny(warnings)]

#[cfg(target_os = "redox")]
extern crate syscall;
#[cfg(target_os = "redox")]
extern crate extra;

extern crate clap;
extern crate libc;

use std::io::{self, Write, Read};
use std::process::exit;
use std::fmt::Write as FmtWrite;
use std::mem;
use std::ptr;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use clap::{Arg, App};

const MAN_PAGE: &'static str = /* @MANSTART{date} */ r#"
NAME
    uptime - show how long the system has been running

SYNOPSIS
    uptime [ -h | --help]

DESCRIPTION
    Prints the length of time the system has been up.

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3600;
const SECONDS_PER_DAY: u64 = 86400;

fn main() {
   let stdout = io::stdout();
   let mut stdout = stdout.lock();

    let matches = App::new("uptime")
        .version("0.1")
        .author("Jose Narvaez <goyox86@gmail.com>")
        .about("Prints the length of time the system has been up.")
        .arg(Arg::with_name("help")
                    .short("h")
                    .long("help")
                    .help("display this help and exit"))
        .get_matches();

    if matches.is_present("help") {
        stdout.write(MAN_PAGE.as_bytes()).unwrap();
        stdout.flush().unwrap();
        exit(0);
    }

    let mut uptime_str = String::new();

    let uptime = get_uptime().unwrap();
    let uptime_secs = uptime % 60;
    let uptime_mins = (uptime / SECONDS_PER_MINUTE) % 60;
    let uptime_hours = (uptime / SECONDS_PER_HOUR) % 24;
    let uptime_days = (uptime / SECONDS_PER_DAY) % 365;

    let fmt_result;
    if uptime_days > 0 {
        fmt_result = write!(&mut uptime_str, "{}d {}h {}m {}s", uptime_days,
                            uptime_hours, uptime_mins, uptime_secs);
    } else if uptime_hours > 0 {
        fmt_result = write!(&mut uptime_str, "{}h {}m {}s", uptime_hours,
                            uptime_mins, uptime_secs);
    } else if uptime_mins > 0 {
        fmt_result = write!(&mut uptime_str, "{}m {}s", uptime_mins,
                            uptime_secs);
    } else {
        fmt_result = write!(&mut uptime_str, "{}s", uptime_secs);
    }

    if fmt_result.is_err() {
        println!("error: couldn't parse uptime");
    }

    stdout.write(uptime_str.as_bytes()).unwrap();
    stdout.flush().unwrap();
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn get_uptime() -> Result<u64, &'static str> {
    let request = &mut [libc::CTL_KERN, libc::KERN_BOOTTIME] as *mut _ as *mut i32;
    let mut uptime = libc::timeval { tv_sec: 0 , tv_usec: 0 };
    let uptime_ptr: *mut libc::c_void = &mut uptime as *mut _ as *mut libc::c_void;
    let mut uptime_len = &mut (mem::size_of::<libc::timeval>() as libc::size_t);

    let syscall_result = unsafe { libc::sysctl(request, 2, uptime_ptr, uptime_len, ptr::null_mut(), 0) };

    if syscall_result == -1 {
        Err("there was an error getting the uptime")
    } else {
        Ok(now() - uptime.tv_sec as u64)
    }

}

#[cfg(target_os = "redox")]
fn get_uptime() -> Result<u64, &'static str> {
    let mut ts = syscall::TimeSpec::default();
    syscall::clock_gettime(syscall::CLOCK_MONOTONIC, &mut ts).unwrap();

    Ok(ts.tv_sec)
}

#[cfg(target_os = "linux")]
fn get_uptime() -> Result<u64, &'static str> {
    let mut proc_uptime = String::new();

    if let Some(n) =
        File::open("/proc/uptime").ok()
            .and_then(|mut f| f.read_to_string(&mut proc_uptime).ok())
            .and_then(|_| proc_uptime.split_whitespace().next())
            .and_then(|s| s.parse::<f64>().ok()) {
                Ok(n as u64)
            } else {
                Err("there was an error getting the uptime")
            }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
