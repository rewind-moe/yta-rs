use std::path::PathBuf;

fn main() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/watchpage_scheduled.html");

    let html = std::fs::read_to_string(d).unwrap();

    println!(
        "{:#?}",
        yta_rs::player_response::get_initial_player_response(&html).unwrap()
    );
}
