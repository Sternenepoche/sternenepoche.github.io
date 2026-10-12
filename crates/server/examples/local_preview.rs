//! Isolated local UI test fixture. It never opens an existing online world.
use sternenepoche_server::local::LocalSession;
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("Testverzeichnis angeben"));
    let bots = std::env::args()
        .nth(2)
        .and_then(|n| n.parse().ok())
        .unwrap_or(30);
    let host = std::env::args().nth(3).as_deref() == Some("host");
    let session = LocalSession::start(&root, bots, host, 0, "demo-offline-master").unwrap();
    println!("{}", session.url);
    std::fs::write(root.join("adresse.txt"), &session.url).unwrap();
    while !root.join("stop.txt").exists() {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
