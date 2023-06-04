use not_so_human_panic::setup_panic;

fn main() {
    setup_panic!();

    println!("A normal log message");
    panic!("OMG EVERYTHING IS ON FIRE!!!");
}
