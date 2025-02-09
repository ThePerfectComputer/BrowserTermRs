use shared::term::{TerminalDownMsg, TerminalScreen, TerminalUpMsg};
use shared::UpMsg;
use zoon::*;

static TERMINAL_STATE: Lazy<Mutable<TerminalDownMsg>> =
    Lazy::new(|| Mutable::new(TerminalDownMsg::TermNotStarted));

pub fn msg_handler(msg: TerminalDownMsg) {
    TERMINAL_STATE.set(msg);
}

pub fn root() -> impl Element {
    term_request();
    let terminal = El::new()
        .s(Width::fill())
        .s(Height::fill())
        .s(Font::new().family([
            FontFamily::new("Lucida Console"),
            FontFamily::new("Courier"),
            FontFamily::new("monospace"),
        ]))
        .update_raw_el(|raw_el| {
            raw_el.global_event_handler(|event: events::KeyDown| {
                send_char((&event).key().as_str(), (&event).ctrl_key());
            })
        })
        .child_signal(
            TERMINAL_STATE
                .signal_cloned()
                .map(|down_msg| match down_msg {
                    TerminalDownMsg::FullTermUpdate(term) => make_grid_with_newlines(&term),
                    TerminalDownMsg::TermNotStarted => "Term not yet started!".to_string(),
                    TerminalDownMsg::BackendTermStartFailure(msg) => {
                        format!("Error: BackendTermStartFailure: {}", msg)
                    }
                }),
        );
    let root = Column::new()
        .s(Width::fill())
        .s(Height::fill())
        .s(Align::new().top())
        .item(terminal);
    root
}

fn term_request() {
    Task::start(async {
        let up_msg = UpMsg::TerminalUpMsg(TerminalUpMsg::RequestFullTermState);
        let _ = crate::CONNECTION.send_up_msg(up_msg.clone()).await;
    });
}

fn send_char(s: &str, has_control: bool) {
    match process_str(s, has_control) {
        Some(c) => {
            Task::start(async move {
                let up_msg = UpMsg::TerminalUpMsg(TerminalUpMsg::SendCharacter(c));
                let _ = crate::CONNECTION.send_up_msg(up_msg).await;
            });
        }
        None => {
            // eprintln!("Not processing: {}", s)
        }
    }
}

fn make_grid_with_newlines(term: &TerminalScreen) -> String {
    let mut formatted = String::new();
    for (i, c) in term.content.chars().enumerate() {
        formatted.push(c);
        if (i + 1) % term.cols == 0 {
            formatted.push('\n');
        }
    }
    formatted
}

fn process_str(s: &str, has_ctrl: bool) -> Option<char> {
    match s {
        "Enter" => {
            return Some('\n');
        }
        "Escape" => {
            return Some('\x1B');
        }
        "Backspace" => {
            return Some('\x08');
        }
        "ArrowUp" => {
            return Some('\x10');
        }
        "ArrowDown" => {
            return Some('\x0E');
        }
        "ArrowLeft" => {
            return Some('\x02');
        }
        "ArrowRight" => {
            return Some('\x06');
        }
        _ => {}
    }
    // Check if the string has exactly one character
    if s.chars().count() == 1 {
        // Safe unwrap because we know the length is 1
        let c = s.chars().next().unwrap();
        let c = process_for_ctrl_char(c, has_ctrl);
        return Some(c);
    }
    None
}

fn is_lowercase_alpha(c: char) -> bool {
    char_is_between_inclusive(c, 'a', 'z')
}

fn process_for_ctrl_char(c: char, has_ctrl: bool) -> char {
    let mut final_ctrl_char = c;
    if has_ctrl {
        if is_lowercase_alpha(c) {
            let c_u8 = c as u8;
            let ctrl_char_u8 = c_u8 - 96;
            final_ctrl_char = ctrl_char_u8 as char;
        } else if char_is_between_inclusive(c, '[', '_') {
            let c_u8 = c as u8;
            let ctrl_char_u8 = c_u8 - 90;
            final_ctrl_char = ctrl_char_u8 as char;
        }
    }
    return final_ctrl_char;
}

fn char_is_between_inclusive(c: char, lo_char: char, hi_char: char) -> bool {
    c >= lo_char && c <= hi_char
}
