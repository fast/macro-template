use macro_template::template;

fn main() {
    template! {
        for (Name, Variant) in [
            (FirstEnum, First),
            (SecondEnum, Second),
        ] {
            enum Name {
                @splice { Variant, }
            }
        }
    }
}
