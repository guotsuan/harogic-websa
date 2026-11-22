//! Windows support for downloading already encoded segments.
use super::*;

const UNAVAILABLE: &str = "Opus encoding is unavailable on Windows";

pub(super) fn unavailable_reason() -> &'static str {
    UNAVAILABLE
}

pub(crate) fn available() -> bool {
    false
}

pub(super) enum Encoder {}

impl Encoder {
    pub(super) fn new(_root: &FsPath, _id: &str, _row: &Value, _frames: u64) -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::Unsupported, UNAVAILABLE))
    }

    pub(super) fn live(root: &FsPath, id: &str, row: &Value) -> io::Result<Self> {
        Self::new(root, id, row, 0)
    }

    pub(super) fn push(&mut self, _pcm: &[u8]) -> io::Result<()> {
        match *self {}
    }
    pub(super) fn finish(self, _cancel: &AtomicBool) -> io::Result<Value> {
        match self {}
    }
}

pub(crate) async fn export(
    State(app): State<Arc<App>>,
    Path(task): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if !params.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if super::audio_files::task_is_direct(&app, &task) {
        return super::audio_files::export(app, task, Some("opus")).await;
    }
    (StatusCode::SERVICE_UNAVAILABLE, UNAVAILABLE).into_response()
}
