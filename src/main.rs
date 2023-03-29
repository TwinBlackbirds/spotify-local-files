use std::error::Error;
use std::io::{Write, stdin, stdout};
use log::{debug, error, log_enabled, info, Level};
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;



fn pull_url(requested_title: &String) -> Result<(), Box<dyn Error>> {
    debug!("Opening browser..");
    let browser = Browser::default()?;

    let tab = browser.new_tab()?;
    debug!("Navigating..");
    let invalids: Vec<(char, &str)> = Vec::from([
        (',', "%2C"), 
        ('?', "%3F"), 
        ('&', "%26"), 
        ('/', "%2F"), 
        ('+', "%3D"), 
        ('=', "%2B"), 
        ('$', "%24"), 
        ('@', "%40"), 
        (' ', "+") // must come last, or at least before the + check
    ]);
    let mut sanitized_search: String = requested_title.clone();
    for i in 1..invalids.len() {
        if sanitized_search.contains(invalids[i].0) {
            sanitized_search = sanitized_search.replace(invalids[i].0, invalids[i].1)
        }
    }
    let search_url: String = String::from("https://www.youtube.com/results?search_query=") + &sanitized_search;
    tab.navigate_to(&search_url)?;
    debug!("Arrived at youtube search page. Waiting for navigation..");
    match tab.wait_until_navigated() {
        Ok(_) => {},
        Err(e) => {error!("Error waiting for navigation: {}", e)}
    }

    
    let _png_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        None,
        None,
        true)?;

    std::fs::write("temp.png", _png_data).unwrap();
    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut requested_title: String = String::new();
    print!("Please enter the song & artist name you would like to download: ");
    match stdout().flush() {
        Ok(()) => {},
        Err(e) => {panic!("Error flushing stdout: {}", e)}
    }
    match stdin().read_line(&mut requested_title) {
        Ok(_) => {},
        Err(e) => {panic!("Error reading input: {}", e)}
    }
    pull_url(&requested_title)?;
    Ok(())
}