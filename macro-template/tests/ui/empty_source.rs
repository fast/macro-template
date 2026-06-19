use macro_template::template;

fn main() {
    template! {
        for T in [] {
            let _ = stringify!(T);
        }
    }
}
