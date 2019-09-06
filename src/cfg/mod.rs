use addr::DomainName;
use std::io;

pub fn init_cfg() {
    // Set domain used
    println!("Enter domain that will be used (with subdomain if used & no http(s)):");
    let mut domain_input = String::new();
    io::stdin()
        .read_line(&mut domain_input)
        .expect("unable to read domain input");

    let domain: DomainName = domain_input.trim().parse().unwrap();
    let domain_root = domain.root().to_string();
    let domain_full = domain.to_string();

    // Check if will be using https
    let https: String;
    'main: loop {
        println!(
            "Is your domain setup to use https (eg. reverse proxy; this does not serve ssl)?[y/n]:"
        );
        let mut https_string = String::new();
        io::stdin()
            .read_line(&mut https_string)
            .expect("unable to read https input");
        let https_ref = https_string.trim();
        if https_ref == "y" || https_ref == "yes" {
            https = "https".to_string();
            break 'main;
        } else if https_ref == "n" || https_ref == "no" {
            https = "http".to_string();
            break 'main;
        } else {
            println!("Not a valid entry. (valid: y/n/yes/no");
        }
    }


}

fn parse_bool(resp: &String) -> bool {
    true
}

fn load_cfg() {}
