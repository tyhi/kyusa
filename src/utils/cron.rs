pub async fn init() {
    println!("hi from cron thread");
    println!("sleeping for 5 seconds");
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("hi im awake!");
}
