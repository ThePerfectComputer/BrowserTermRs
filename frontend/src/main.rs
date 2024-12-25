use zoon::*;
use zoon::eprintln;
use std::rc::Rc;

mod term;

pub static WINDOW_SIZE: Lazy<Mutable<u32>> = Lazy::new(|| Mutable::new(0));

fn root() -> impl Element {
    El::new()
        .s(Width::fill().max(800))
        .s(Height::fill().max(450))
        // center
        .s(Align::center())
        .child(term::root())

}

fn main() {
    start_app("app", root);
}
