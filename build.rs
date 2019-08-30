fn main() {
    println!("yeehaw");
    built::write_built_file().expect("Failed to acquire build-time information");
}
