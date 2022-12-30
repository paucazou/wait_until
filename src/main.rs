use chrono;
use chrono::Datelike;
use chrono::Duration;
use chrono::LocalResult;
//use chrono::naive;
use chrono::TimeZone;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser = validate_hour)]
    target_hour: chrono::DateTime<chrono::Local>,
}

use std::io;
use std::io::Write;
use std::thread;
use std::time;

use terminal_size::*;


fn validate_hour (s: &str)-> Result<chrono::DateTime<chrono::Local>, String> {
    // Split the input string by the first occurrence of ':'
    let parts: Vec<&str> = s.splitn(3, ':').collect();

    // Check the number of parts in the input string
    let (hour, minute, second) = match parts.len() {
        // If there is only one part, it represents the hour
        1 => (parts[0], "0", "0"),
        // If there are two parts, they represent the hour and minute
        2 => (parts[0], parts[1], "0"),
        // If there are three parts, they represent the hour, minute, and second
        3 => (parts[0], parts[1], parts[2]),
        // If there are more than three parts, return an error
        _ => return Err("Invalid input format".to_string()),
    };

    // Parse the hour, minute, and second parts
    let hour: u32 = match hour.parse() {
        Ok(h) => h,
        Err(_) => return Err("Invalid hour".to_string()),
    };
    let minute: u32 = match minute.parse() {
        Ok(m) => m,
        //Err(_) => return Err("Invalid minute".to_string()),
        Err(e) => return Err(format!("Invalid minute: {e}").to_string()),
    };
    let second: u32 = match second.parse() {
        Ok(s) => s,
        Err(_) => return Err("Invalid second".to_string()),
    };

    let start = chrono::Local::now();
    let mut target_hour = match chrono::Local.with_ymd_and_hms(start.year(), start.month(), start.day(), hour, minute, second) {
        LocalResult::Single(elt) => elt,
        LocalResult::Ambiguous(_, _) => return Err("Ambiguous hour".to_string()),
        LocalResult::None => return Err("No datetime can be created".to_string()),
    };
    // is target hour before current hour?
    if target_hour <= start {
        target_hour += Duration::days(1);
    }
    Ok(target_hour)
}

/// Written by ChatGpt because I'm too lazy for that kind of things
/// (but with so many errors that I should have done it myself)
/// Return a string representation of a chrono::Duration
fn format_duration(duration: &Duration) -> String {
    let mut result = String::new();

    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let format_space = |result: &String| if result.len() != 0 { " " } else { "" };

    if days > 0 {
        result.push_str(&format!("{}d", days));
    }
    if hours > 0 {
        result.push_str(&format!("{}{}h",format_space(&result), hours));
    }
    if minutes > 0 {
        result.push_str(&format!("{}{}m",format_space(&result), minutes));
    }
    if seconds > 0 {
        result.push_str(&format!("{}{}s", format_space(&result), seconds));
    } 

    result
}


fn main() {
    // arg parser
    let cli = Cli::parse();
    let start = chrono::Local::now();
    let target_hour = cli.target_hour;
    // duration
    let total_duration = target_hour - start;
    let mut time_to_wait = total_duration;

    // necessary to flush
    let mut stdout = io::stdout();

    println!("Waiting for {} until {}", format_duration(&total_duration), target_hour.format("%d/%m/%Y %H:%M:%S"));

    // main loop
    while time_to_wait >= Duration::seconds(0) {
        // get terminal size
        let width = match terminal_size() {
            Some((Width(w), Height(_))) => w,
            None => panic!("No terminal size found"),
        };

        let elapsed_time = chrono::Local::now() - start;
        // percent part
        let elapsed_percent: u16 = (elapsed_time.num_seconds() * 100 / total_duration.num_seconds()) as u16;
        let elapsed_percent_string = elapsed_percent.to_string();
        // duration left
        let time_to_wait_string = format_duration(&time_to_wait);
        // size of one second
        const EXTRA_CHAR_NB: usize = 6; // ie: [>] % and spaces
        let total_bar = width as usize - EXTRA_CHAR_NB - elapsed_percent_string.len() - time_to_wait_string.len();
        let sec_size: f64= total_bar as f64 / total_duration.num_seconds() as f64;
        // bar
        let bar_size = (sec_size * elapsed_time.num_seconds() as f64) as i32;
        let spaces_size = total_bar - bar_size as usize;
        let bar_line = "=".repeat(bar_size as usize);
        let spaces_line = " ".repeat(spaces_size as usize);

        // \r or \n?
        let eol = if time_to_wait.num_seconds() == 0 { "\n" } else { "\r" };
        // print
        print!("[{bar_line}>{spaces_line}] {elapsed_percent_string}% {time_to_wait_string}{eol}");
        stdout.flush().unwrap(); // yes, unwrap is bad, but for this case, it's good
        thread::sleep(time::Duration::from_millis(1000));
        time_to_wait = target_hour - chrono::Local::now();



    }


}


#[test]
fn test_format_duration() {
    let duration = Duration::seconds(3662);
    let expected = "1h 1m 2s";
    assert_eq!(format_duration(&duration), expected);

    let duration = Duration::hours(1);
    let expected = "1h";
    assert_eq!(format_duration(&duration), expected);

    let duration = Duration::seconds(60);
    let expected = "1m";
    assert_eq!(format_duration(&duration), expected);
    
    let duration = Duration::seconds(70);
    let expected = "1m 10s";
    assert_eq!(format_duration(&duration), expected);

    let duration = Duration::seconds(1);
    let expected = "1s";
    assert_eq!(format_duration(&duration), expected);

    let duration = Duration::seconds(0);
    let expected = "";
    assert_eq!(format_duration(&duration), expected);
}

#[test]
fn test_validate_hour() {
    let start = chrono::Local::now();
    let gen_today = | h: u32, m: u32, s: u32 | chrono::Local.with_ymd_and_hms(start.year(), start.month(), start.day(), h, m, s).unwrap();
    let gen_tomorrow_if_too_late = | dt: chrono::DateTime::<chrono::Local> | if dt < start { dt + chrono::Duration::days(1) } else { dt};
    let gen_time = | h: u32, m: u32, s: u32 | gen_tomorrow_if_too_late(gen_today(h, m, s));
    let input_expected = vec!(
        ("19",gen_time(19,0,0)),
        ("01",gen_time(1,0,0)),
        ("2",gen_time(2,0,0)),
        ("20:03",gen_time(20,3,0)),
        ("20:4",gen_time(20,4,0)),
        ("0:4",gen_time(0,4,0)),
        ("19:40:05",gen_time(19,40,5)),
        );
    for elt in input_expected {
        assert_eq!(validate_hour(elt.0), Ok(elt.1));
    }
}
