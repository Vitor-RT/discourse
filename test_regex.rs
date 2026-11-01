use regex::Regex;

fn main() {
    let filename = "backup-2023-10-25-010101-v-2023-10-25-12345678901234.tar.gz";
    let pattern = r"-(v)-(?:(?P<discourse_version>\d{4}-\d{1,2}-\d+(?:-latest(?:-\d+)?)?)-)?(?P<database_version>\d{14})";
    let re = Regex::new(&pattern).unwrap();
    if let Some(caps) = re.captures(filename) {
        println!("caps: {:?}", caps);
        println!("discourse_version: {:?}", caps.name("discourse_version"));
    } else {
        println!("no match");
    }
}
