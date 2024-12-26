use moonlight::*;

pub mod term;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "serde")]
pub enum UpMsg {
    TerminalUpMsg(term::TerminalUpMsg),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "serde")]
pub enum DownMsg {
    TerminalDownMsg(term::TerminalDownMsg),
}
