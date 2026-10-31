//! XIM 서버. X 세션에 `@im=hangul` 입력기로 붙어 앱의 키 입력을 엔진에 넘긴다.

mod xim_handler;

use hangul_setting::keymap::Keymap;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::ConnectionExt;
use xim::x11rb::X11rbServer;
use xim::{Request, XimConnections};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("hangul-setting {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let (conn, screen_num) = x11rb::rust_connection::RustConnection::connect(None)?;
    let mut server = X11rbServer::init(&conn, screen_num, IM_NAME, xim::ALL_LOCALES)?;
    let mut connections = XimConnections::new();
    let mut handler = Handler::new(load_keymap(&conn)?);
    log::info!("XIM 서버 시작: XMODIFIERS=@im={IM_NAME}");

    loop {
        let event = conn.wait_for_event()?;
        match &event {
            Event::MappingNotify(_) => handler.set_keymap(load_keymap(&conn)?),
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
