use moon::*;
use shared::term::{TerminalDownMsg, TerminalScreen};
use shared::term::{TerminalUpMsg};
use shared::{DownMsg};

use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use std::collections::HashMap;

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::event::Notify;
use alacritty_terminal::event_loop::{EventLoop, Notifier};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{self, Term as Terminal};
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::{tty, Grid};

use crate::terminal_size;

static SESSIONS_V_TERM_SESSION: Lazy<RwLock<HashMap<SessionId, Arc<RwLock<TerminalSession>>>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

#[derive(Clone)]
struct EventProxy(mpsc::Sender<Event>);
impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let sender = self.0.clone();
        if let Err(err) = sender.blocking_send(event) {
            eprintln!("Failed to send event: {:?}", err);
        }
    }
}

struct TerminalSession {
    term: Arc<FairMutex<Terminal<EventProxy>>>,
    /// Use to write to the terminal from the outside world.
    tx : Notifier,
    cols: u16,
    rows: u16
}

pub async fn up_msg_handler(
    msg:        TerminalUpMsg,
    session_id: SessionId,
    cor_id:     CorId,
    auth_token: Option<AuthToken>
    ) {
    match sessions::by_session_id().wait_for(session_id).await {
        Some(session) => {
            if term_session_exists(session_id) {
                let term_session = SESSIONS_V_TERM_SESSION.read().unwrap();
                let term_session = term_session.get(&session_id).unwrap();
                let term_session = term_session.read().unwrap();

                match msg {
                    TerminalUpMsg::RequestFullTermState => {
                        let down_msg = DownMsg::TerminalDownMsg(
                            TerminalDownMsg::FullTermUpdate(TerminalScreen {
                                rows: term_session.rows as usize,
                                cols: term_session.cols as usize,
                                content: terminal_session_to_string(&*term_session)
                            })
                        );
                        session.send_down_msg(&down_msg, cor_id).await;
                    }
                    TerminalUpMsg::SendCharacter(c) => {
                        term_session.tx.notify(c.to_string().into_bytes());

                    }
                    _ => {}
                }

            }
            else {
                let (rows, cols) = (24, 80);

                let id = get_session_count() as u64;
                let pty_config = tty::Options {
                    shell: Some(tty::Shell::new("/bin/bash".to_string(), vec![])),
                    ..tty::Options::default()
                };
                let config = term::Config::default();
                let terminal_size = terminal_size::TerminalSize::new(rows, cols);
                let pty = match tty::new(&pty_config, terminal_size.into(), id) {
                    Ok(pty) => pty,
                    Err(_) => {
                        backend_term_start_error(session, cor_id, "tty::new failed".to_string()).await;
                        return;
                    }
                };
                let (event_sender, mut event_receiver) = mpsc::channel(100);
                let event_proxy = EventProxy(event_sender);
                let term = Terminal::new::<terminal_size::TerminalSize>(
                        config,
                        &terminal_size.into(),
                        event_proxy.clone(),
                    );
                let term = Arc::new(FairMutex::new(term));
                let pty_event_loop = match EventLoop::new(term.clone(), event_proxy, pty, false, false) {
                    Ok(loop_instance) => loop_instance,
                    Err(err) => {
                        backend_term_start_error(
                            session,
                            cor_id,
                            format!("EventLoop::new failed: {:?}", err),
                        )
                        .await;
                        return;
                    }
                };
                let notifier = Notifier(pty_event_loop.channel());
                let term_clone = term.clone();
                pty_event_loop.spawn();

                tokio::spawn(handle_event(
                    event_receiver,
                    term_clone,
                    rows,
                    cols,
                    session.clone(),
                    cor_id,
                ));

                let terminal_session = TerminalSession {
                    term: term.clone(),
                    tx  : notifier,
                    cols: cols,
                    rows: rows
                };
                SESSIONS_V_TERM_SESSION.write().unwrap().insert(
                    session_id,
                    Arc::new(RwLock::new(terminal_session))
                );

            }
        }
        None => {
            eprintln!("cannot find the session with id `{}`", session_id);

        }

    }

}

async fn backend_term_start_error(
    session: SessionActor,
    cor_id:     CorId,
    msg : String,
) {
    let down_msg = DownMsg::TerminalDownMsg(
        TerminalDownMsg::BackendTermStartFailure(msg));
    session.send_down_msg(&down_msg, cor_id).await;
}
fn get_session_count() -> usize {
    let sessions_map = SESSIONS_V_TERM_SESSION.read().unwrap();
    sessions_map.len()
}

fn term_session_exists(session_id: SessionId) -> bool {
    let sessions_map = SESSIONS_V_TERM_SESSION.read().unwrap();
    sessions_map.contains_key(&session_id)
}

fn terminal_session_to_string(terminal_session: &TerminalSession) -> String {
    let (rows, cols) = (terminal_session.rows, terminal_session.cols);
    let term = terminal_session.term.lock();
    let grid = term.grid().clone();

    return term_grid_to_string(&grid, rows, cols);
}

fn term_grid_to_string(grid: &Grid<Cell>, rows: u16, cols: u16) -> String {
    let mut term_content = String::with_capacity((rows*cols) as usize);

    // Populate string from grid
    for indexed in grid.display_iter() {
        let x = indexed.point.column.0 as usize;
        let y = indexed.point.line.0 as usize;
        if y < rows as usize && x < cols as usize {
            term_content.push(indexed.c);
        }
    }
    return term_content;
}

async fn handle_event(
    mut event_receiver: mpsc::Receiver<Event>,
    term: Arc<FairMutex<Terminal<EventProxy>>>,
    rows: u16,
    cols: u16,
    session: SessionActor,
    cor_id: CorId,
) {
    loop {
        if let Some(event) = event_receiver.recv().await {
            match event {
                Event::Exit => {
                    break;
                }
                _ => {
                    let grid = term.lock().grid().clone();
                    let term_content = term_grid_to_string(&grid, rows, cols);
                    let down_msg = DownMsg::TerminalDownMsg(
                        TerminalDownMsg::FullTermUpdate(TerminalScreen {
                            rows: rows as usize,
                            cols: cols as usize,
                            content: term_content,
                        }),
                    );
                    session.send_down_msg(&down_msg, cor_id).await;
                }
            }
        }
    }
}
