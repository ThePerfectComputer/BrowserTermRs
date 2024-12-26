use shared::{DownMsg, UpMsg};
use zoon::*;

mod term;

pub static WINDOW_SIZE: Lazy<Mutable<u32>> = Lazy::new(|| Mutable::new(0));

pub static CONNECTION: Lazy<Connection<UpMsg, DownMsg>> = Lazy::new(|| {
    Connection::new(|down_msg, _| match down_msg {
        DownMsg::TerminalDownMsg(terminal_msg) => term::msg_handler(terminal_msg),
    })
});

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
