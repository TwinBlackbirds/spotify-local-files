use std::error::Error;
use std::path::Path;
use std::io::{Write, stdin, stdout};
use log::{debug, error};
use headless_chrome::{protocol::cdp::Page, Browser, LaunchOptionsBuilder};

struct Video {
    title: String,
    channel: String,
    length: String,
    link: String
}

struct CharPair<'a> {
    original: char,
    replacement: &'a str,
}

fn pull_urls(requested_title: &String) -> Result<Vec<Video>, Box<dyn Error>> {
    debug!("Opening browser..");
    let mut builder = LaunchOptionsBuilder::default();
    builder.headless(true);
    builder.window_size(Some((1920, 1080)));
    let launch_options = builder.build()?;
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    
    debug!("Navigating..");
    let invalids_list: Vec<CharPair> = Vec::from([
        CharPair{original: ',', replacement: "%2C"},
        CharPair{original: '?', replacement: "%3F"},
        CharPair{original: '&', replacement: "%26"}, 
        CharPair{original: '/', replacement: "%2F"},
        CharPair{original: '+', replacement: "%3D"},
        CharPair{original: '=', replacement: "%2B"},
        CharPair{original: '$', replacement: "%24"},
        CharPair{original: '@', replacement: "%40"},
        CharPair{original: ' ', replacement: "+"}, // must come last, or at least before the + check
    ]); 
    
    let mut sanitized_search: String = requested_title.clone();
    for invalid in invalids_list {
        if sanitized_search.contains(invalid.original) {
            sanitized_search = sanitized_search.replace(invalid.original, invalid.replacement);
        }
    }
    let search_url: String = String::from("https://www.youtube.com/results?search_query=") + &sanitized_search;
    tab.navigate_to(&search_url)?;
    debug!("Arrived at youtube search page. Waiting for navigation..");
    match tab.wait_until_navigated() {
        Ok(_) => {},
        Err(e) => {error!("Error waiting for navigation: {}", e)}
    }
    let mut valid_urls_pulled: Vec<Video> = Vec::new();
    let mut valid_urls: i8 = 0;
    
    match tab.wait_for_elements("ytd-video-renderer > div[id=\"dismissible\"]") {
        Ok(video_parent_els) => {
            for i in 0..video_parent_els.len() {
                if valid_urls > 4 {
                    break
                }
                let video_length_el = video_parent_els[i].find_element(
                    "ytd-thumbnail > a[id=\"thumbnail\"] > div[id=\"overlays\"] > 
                    ytd-thumbnail-overlay-time-status-renderer > span")?;
                let mut video_length = String::new();
                match video_length_el.call_js_fn(
                    "function() { return this.innerHTML; }", 
                    vec![], true)?.value {
                        Some(v_length) => {
                            // i think this is more optimized than replacing the string 4 times
                            for i in v_length.to_string().as_bytes() {
                                let cast: char = *i as char;
                                if cast.is_ascii_digit() || cast == ':' {
                                    video_length.push(cast)
                                }
                            }
                        },
                        None => {println!("No video length element found for video {}", i)}
                    };
                let text_wrapper_el = 
                video_parent_els[i].find_element("div[class*=\"text-wrapper\"]")?;
                let mut video_title: String = String::new();
                let video_title_el = text_wrapper_el.find_element("a[id=\"video-title\"]")?;
                match video_title_el.call_js_fn(
                    "function() { return this.title; }",
                    vec![], true)?.value {
                    Some(title_val) => {video_title = title_val.to_string()},
                    None => {println!("No video title element found on video {}", i)}
                }
                let channel_el = text_wrapper_el.find_element("ytd-channel-name[id=\"channel-name\"]")?;
                let mut href: String = String::new();
                match video_title_el.call_js_fn(
                    "function() { return this.href; }", 
                    vec![], true)?.value {
                    Some(href_val) => {href = href_val.to_string()},
                    None => {println!("No video href element found on video {}", i)}
                }
                let channel_name_el = channel_el.find_element("#text .yt-formatted-string")?;
                let mut channel_name: String = String::new();
                match channel_name_el.call_js_fn(
                    "function() { return this.innerHTML; }", 
                    vec![], true)?.value {
                        Some(channel_val) => {channel_name = channel_val.to_string()},
                        None => {println!("No channel name element found on video {}", i)}
                }
                if !href.contains("shorts") { // filter out shitty youtube shorts poopoo caca
                    let video = Video {
                        title: video_title, 
                        channel: channel_name, 
                        length: video_length,
                        link: href
                    };
                    valid_urls_pulled.push(video);
                    valid_urls += 1;
                } 
            }
        }
        Err(e) => {panic!("Error grabbing urls: {}", e)}
    }
    
    // debug screenie
    let _png_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        None,
        None,
        true)?;

    std::fs::write("preview.png", _png_data)?;
    Ok(valid_urls_pulled)
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
    requested_title = requested_title.trim().to_string();
    println!("Ok, gathering video options..");
    let urls_list: Vec<Video> = pull_urls(&requested_title)?;
    println!("Top 5 YouTube results (viewable in preview.png):");
    for i in 0..urls_list.len() {
        println!("{}. {} uploaded by {} (duration {})", i+1, 
        urls_list[i].title, urls_list[i].channel, urls_list[i].length)
    }
    let mut requested_video: String = String::new();
    print!("Please choose which video to download (1-5): ");
    match stdout().flush() {
        Ok(()) => {},
        Err(e) => {panic!("Error flushing stdout: {}", e)}
    }
    match stdin().read_line(&mut requested_video) {
        Ok(_) => {},
        Err(e) => {panic!("Error reading input: {}", e)}
    }
    let parsed_input: u32 = match requested_video.chars().nth(0) {
        Some(val) => {match val.to_digit(10) {
            Some(val) => {val},
            None => { println!("Enter a valid input!"); std::process::exit(0) }
        }},
        None => { println!("Enter a valid input!"); std::process::exit(0) }
    };
    // add extra quotes to chosen_link for yt-dlp (workaround for literal symbols interpreted by bash)
    let chosen_link: String = urls_list[parsed_input as usize - 1].link.replace("\"", "");
    debug!("DBG: Using url: {}", chosen_link);
    let mut requested_filename: String = String::new();
    print!("Please provide a name for the file to be stored as (no extension): ");
    match stdout().flush() {
        Ok(_) => {}
        Err(e) => {panic!("Error while flushing stdout: {}", e)}
    }
    match stdin().read_line(&mut requested_filename) {
        Ok(_) => {},
        Err(e) => {panic!("Error reading input: {}", e)}
    }
    requested_filename = requested_filename.trim().to_string();
    println!("Downloading..");
    if Path::new("current.mp4").exists() {
        std::fs::remove_file("current.mp4")?;
    }
    let mut dl_cmd = std::process::Command::new("./yt-dlp");
    dl_cmd.args([&chosen_link, "-f", "best[ext=mp4]", "-o", "current.mp4"]);
    dl_cmd.output().expect("Failed to download video!");
    let mut conv_cmd = std::process::Command::new("ffmpeg");
    conv_cmd.args(["-i", "current.mp4", &(format!("{}.mp3", &requested_filename))]);
    conv_cmd.output().expect("Failed to convert video to mp3!");
    println!("File saved as ./{}.mp3", requested_filename);
    println!("Cleaning up..");
    if Path::new("current.mp4").exists() {
        std::fs::remove_file("current.mp4")?;
    }
    if Path::new("preview.png").exists() {
        std::fs::remove_file("preview.png")?;
    }
    println!("All done! Exiting..");
    Ok(())
}