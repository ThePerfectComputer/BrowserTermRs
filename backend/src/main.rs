use moon::*;
use shared::{UpMsg};

use std::sync::Mutex;

mod term;
mod terminal_size;

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Joy Of Hardware")
        .append_to_head(include_str!("../favicon.html"))
        .append_to_head(
            "
        <style>
            html {
                background-color: white;
            }
        </style>",
        )
}

async fn up_msg_handler(req: UpMsgRequest<UpMsg>) {
    let UpMsgRequest { up_msg, session_id, cor_id, auth_token } = req;
    let up_msg_clone = up_msg.clone();

    match up_msg {
        UpMsg::TerminalUpMsg(terminal_up_msg) =>
        {
            term::up_msg_handler(
                terminal_up_msg,
                session_id,
                cor_id,
                auth_token).await
        }
    }

}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
