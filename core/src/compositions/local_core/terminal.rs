//! Port of `backend/compositions/local-core/terminal.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.TerminalService/CreateTerminalSession",
            create_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/WriteTerminalSession",
            write_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/ResizeTerminalSession",
            resize_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/CloseTerminalSession",
            close_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/ReadTerminalOutput",
            read_terminal_output,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/ListTerminalSessions",
            list_terminal_sessions,
        ),
    ]
}

/// `polterware.local.v1.TerminalService.CreateTerminalSession`, not ported yet.
pub fn create_terminal_session(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CreateTerminalSessionRequest>,
) -> Result<local_v1::CreateTerminalSessionResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.TerminalService.CreateTerminalSession is not implemented",
    ))
}

/// `polterware.local.v1.TerminalService.WriteTerminalSession`, not ported yet.
pub fn write_terminal_session(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::WriteTerminalSessionRequest>,
) -> Result<local_v1::WriteTerminalSessionResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.TerminalService.WriteTerminalSession is not implemented",
    ))
}

/// `polterware.local.v1.TerminalService.ResizeTerminalSession`, not ported yet.
pub fn resize_terminal_session(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ResizeTerminalSessionRequest>,
) -> Result<local_v1::ResizeTerminalSessionResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.TerminalService.ResizeTerminalSession is not implemented",
    ))
}

/// `polterware.local.v1.TerminalService.CloseTerminalSession`, not ported yet.
pub fn close_terminal_session(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CloseTerminalSessionRequest>,
) -> Result<local_v1::CloseTerminalSessionResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.TerminalService.CloseTerminalSession is not implemented",
    ))
}

/// `polterware.local.v1.TerminalService.ReadTerminalOutput`, not ported yet.
pub fn read_terminal_output(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ReadTerminalOutputRequest>,
) -> Result<local_v1::ReadTerminalOutputResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.TerminalService.ReadTerminalOutput is not implemented",
    ))
}

/// `polterware.local.v1.TerminalService.ListTerminalSessions`, not ported yet.
pub fn list_terminal_sessions(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListTerminalSessionsRequest>,
) -> Result<local_v1::ListTerminalSessionsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.TerminalService.ListTerminalSessions is not implemented",
    ))
}
