use discourse_core::types::base62;
use discourse_core::types::color_math;
use discourse_core::text_utils::discourse_diff::DiscourseDiff;
use discourse_core::browser::crawler_detection::CrawlerDetection;
use discourse_core::browser::detection::Browser;
use discourse_core::file_store::file_helper::FileHelper;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: parity_runner <command> [args]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "base62_encode" => {
            let num: u64 = args[2].parse().unwrap();
            println!("{}", base62::encode(num));
        }
        "base62_decode" => {
            println!("{}", base62::decode(&args[2]));
        }
        "color_brightness" => {
            println!("{:.3}", color_math::brightness(&args[2]));
        }
        "diff_inline" => {
            let diff = DiscourseDiff::new(args[2].clone(), args[3].clone());
            println!("{}", diff.inline_html());
        }
        "browser_is_crawler" => {
            let is_bot = CrawlerDetection::is_crawler(Some(&args[2]), None, "", "", "", false);
            println!("{}", is_bot);
        }
        "browser_parse" => {
            println!("{:?}", Browser::parse(&args[2]));
        }
        "file_sanitize_filename" => {
            println!("{}", FileHelper::sanitize_filename(&args[2]));
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
