use crossbeam_channel::Sender;
use time::MusicalTime;

pub struct AudioCommand {
    pub response_sender: Sender<Response>,
    pub request: Request,
}

pub enum Request {
    SetPlayhead(MusicalTime),
    Start,
    Stop,

    GetPlayhead,
}
pub enum Response {
    Ok,
    GetPlayheadResponse(MusicalTime),
}
