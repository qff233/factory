mod dispatch;
mod ping;
mod vehicle;

use dispatch::DispatchExec;
use dispatch::DispatchRequest;
use ping::PingPong;
use tokio::sync::mpsc;

use crate::mcs::track;

pub fn dispatch(
    tool_warn_level: f32,
    track_graph: track::TrackGraph,
) -> (DispatchRequest, DispatchExec) {
    let (sender, receiver) = mpsc::channel(200);
    let request = DispatchRequest::new(sender);
    let exec = DispatchExec::new(receiver, tool_warn_level, track_graph);

    (request, exec)
}
