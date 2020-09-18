pub fn main() {
    cc::Build::new().file("src/stub.S").compile("stub");
}
