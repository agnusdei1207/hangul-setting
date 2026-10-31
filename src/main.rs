//! XIM 서버. X 세션에 `@im=hangul` 입력기로 붙어 앱의 키 입력을 엔진에 넘긴다.

mod ibus;
mod preedit_window;
mod xim_handler;

use std::sync::{Arc, Mutex};

use hangul_setting::config;
use hangul_setting::engine::Engine;
use hangul_setting::keymap::Keymap;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;
use xim::x11rb::X11rbServer;
use xim::{Request, XimConnections};

use crate::preedit_window::PreeditWindow;
use crate::xim_handler::Handler;

/// `XMODIFIERS=@im=hangul`의 이름.
const IM_NAME: &str = "hangul";

fn load_keymap(conn: &impl Connection) -> Result<Keymap, Box<dyn std::error::Error>> {
    let setup = conn.setup();
    let count = setup.max_keycode - setup.min_keycode + 1;
    let reply = conn.get_keyboard_mapping(setup.min_keycode, count)?.reply()?;
    Ok(Keymap::from_mapping(
        setup.min_keycode,
        reply.keysyms_per_keycode,
        &reply.keysyms,
    ))
}

/// X 서버에 붙는다. x11rb는 소켓 파일(`/tmp/.X11-unix/X0`)만 시도하므로, 실패하면 libxcb처럼 리눅스
/// 추상 소켓(`@/tmp/.X11-unix/X0`)으로 다시 붙는다. WSLg처럼 소켓 폴더가 읽기 전용이면 X 서버가
/// 추상 소켓만 연다.
fn connect_x() -> Result<(RustConnection, usize), Box<dyn std::error::Error>> {
    let error = match RustConnection::connect(None) {
        Ok(connection) => return Ok(connection),
        Err(error) => error,
    };
    #[cfg(target_os = "linux")]
    if let Some(connection) = connect_abstract_socket() {
        return Ok(connection);
    }
    Err(error.into())
}

#[cfg(target_os = "linux")]
fn connect_abstract_socket() -> Option<(RustConnection, usize)> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixStream};
    use x11rb::reexports::x11rb_protocol::{parse_display, xauth};
    use x11rb::rust_connection::DefaultStream;

    let display = parse_display::parse_display(None).ok().filter(|d| d.host.is_empty())?;
    let address = SocketAddr::from_abstract_name(format!("/tmp/.X11-unix/X{}", display.display)).ok()?;
    let (stream, (family, peer)) = DefaultStream::from_unix_stream(UnixStream::connect_addr(&address).ok()?).ok()?;
    let (auth_name, auth_data) = xauth::get_auth(family, &peer, display.display)
        .ok()
        .flatten()
        .unwrap_or_default();
    let screen = usize::from(display.screen);
    let connection = RustConnection::connect_to_stream_with_auth_info(stream, screen, auth_name, auth_data).ok()?;
    log::info!("X 서버 추상 소켓으로 연결: @/tmp/.X11-unix/X{}", display.display);
    Some((connection, screen))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("hangul-setting {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let (conn, screen_num) = connect_x()?;
    let mut server = X11rbServer::init(&conn, screen_num, IM_NAME, xim::ALL_LOCALES)?;
    let mut connections = XimConnections::new();
    let config = config::load().unwrap_or_else(|e| {
        log::error!("설정 오류, 기본 한/영 키를 씁니다: {e}");
        config::Config::default()
    });
    let engine = Arc::new(Mutex::new(Engine::new(config.toggle_keys)));
    let ibus_engine = engine.clone();
    // Qt 앱용. 세션 버스가 없어도 XIM 입력은 계속돼야 한다.
    std::thread::spawn(move || {
        if let Err(e) = zbus::block_on(ibus::serve(ibus_engine)) {
            log::warn!("IBus 포털 꺼짐(Qt 앱 입력 불가): {e}");
        }
    });
    let preedit_window = PreeditWindow::new(&conn, screen_num)?;
    let mut handler = Handler::new(engine, load_keymap(&conn)?, preedit_window);
    log::info!("XIM 서버 시작: XMODIFIERS=@im={IM_NAME}");

    loop {
        let event = conn.wait_for_event()?;
        match &event {
            Event::MappingNotify(_) => handler.set_keymap(load_keymap(&conn)?),
            Event::Expose(e) => handler.handle_expose(e.window),
            Event::ClientMessage(msg) => {
                handler.com_win = msg.window;
                // xim 크레이트는 SyncReply를 버리므로 짧은 메시지에서 직접 읽는다.
                if msg.format == 8
                    && let Ok(Request::SyncReply {
                        input_method_id,
                        input_context_id,
                    }) = xim::read(&msg.data.as_data8())
                    && let Err(e) = handler.handle_sync_reply(&mut server, input_method_id, input_context_id)
                {
                    log::warn!("붙잡아 둔 키 보내기 실패: {e}");
                }
            }
            _ => {}
        }
        // 앱 하나가 잘못된 메시지를 보내거나 사라져도 다른 앱의 입력은 계속돼야 한다.
        if let Err(e) = server.filter_event(&event, &mut connections, &mut handler) {
            log::warn!("XIM 이벤트 처리 실패: {e}");
        }
    }
}
