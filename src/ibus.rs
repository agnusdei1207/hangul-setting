//! IBus 포털 D-Bus 인터페이스. Qt에 들어 있는 `ibus` 입력기 플러그인이 붙는다(INTENT-0002).
//!
//! Qt 플러그인은 `IBUS_USE_PORTAL=1`이면 ibus-daemon 없이 세션 버스의
//! `org.freedesktop.portal.IBus`에서 입력 컨텍스트를 만든다. ibus 라이브러리·데몬은 쓰지 않는다.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use hangul_setting::engine::{Engine, Key};
use hangul_setting::hangul::Composer;
use zbus::message::Header;
use zbus::object_server::{ObjectServer, SignalEmitter};
use zbus::{Connection, OwnedObjectPath, Structure, Variant, connection, fdo, interface};

const PORTAL_NAME: &str = "org.freedesktop.portal.IBus";
const IBUS_PATH: &str = "/org/freedesktop/IBus";
/// IBus가 키 놓음에 붙이는 상태 비트.
const RELEASE_MASK: u32 = 1 << 30;
/// 앱이 포커스를 옮길 때 조합 중 글자를 스스로 확정하게 하는 조합 모드(`IBUS_ENGINE_PREEDIT_COMMIT`).
const PREEDIT_COMMIT: u32 = 1;
const ATTR_UNDERLINE: u32 = 1;
const UNDERLINE_SINGLE: u32 = 1;

/// 앱(버스의 고유 이름)마다 만든 입력 컨텍스트. 앱이 Destroy 없이 끝나도 회수한다.
type Owners = Arc<Mutex<HashMap<String, Vec<OwnedObjectPath>>>>;

/// IBus 직렬화 객체 머리: (이름, 첨부 a{sv}) 뒤에 본문이 온다.
fn no_attachments() -> HashMap<String, Variant<'static>> {
    HashMap::new()
}

/// `IBusText` = (sa{sv}sv). 조합 중 글자에는 밑줄 속성을 붙인다.
fn ibus_text(text: &str, underline: bool) -> Variant<'static> {
    let len = text.chars().count() as u32;
    let attributes: Vec<Variant<'static>> = if underline && len > 0 {
        let attr = (
            "IBusAttribute",
            no_attachments(),
            ATTR_UNDERLINE,
            UNDERLINE_SINGLE,
            0u32,
            len,
        );
        vec![Variant::from(Structure::from(attr))]
    } else {
        Vec::new()
    };
    let list = Structure::from(("IBusAttrList", no_attachments(), attributes));
    let text = ("IBusText", no_attachments(), text.to_owned(), Variant::from(list));
    Variant::from(Structure::from(text))
}

struct Portal {
    engine: Arc<Mutex<Engine>>,
    owners: Owners,
    next_id: u64,
}

#[interface(name = "org.freedesktop.IBus.Portal")]
impl Portal {
    async fn create_input_context(
        &mut self,
        _client_name: &str,
        #[zbus(header)] header: Header<'_>,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> fdo::Result<OwnedObjectPath> {
        self.next_id += 1;
        let path = OwnedObjectPath::try_from(format!("{IBUS_PATH}/InputContext_{}", self.next_id))
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let context = InputContext {
            engine: self.engine.clone(),
            composer: Composer::default(),
            ..Default::default()
        };
        server.at(&path, context).await?;
        if let Some(sender) = header.sender() {
            let mut owners = self.owners.lock().expect("owners lock");
            owners.entry(sender.to_string()).or_default().push(path.clone());
        }
        Ok(path)
    }
}

#[derive(Default)]
struct InputContext {
    engine: Arc<Mutex<Engine>>,
    composer: Composer,
    client_commit_preedit: bool,
    content_type: (u32, u32),
}

#[interface(name = "org.freedesktop.IBus.InputContext")]
impl InputContext {
    /// 앱은 응답이 `false`면 원래 키를 스스로 다시 처리한다. 확정 시그널을 응답보다 먼저 보내므로
    /// 조합 중이던 글자가 그 키보다 먼저 들어간다.
    async fn process_key_event(
        &mut self,
        keyval: u32,
        _keycode: u32,
        state: u32,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        if state & RELEASE_MASK != 0 {
            return Ok(false);
        }
        let before = self.composer.preedit();
        let outcome = {
            let mut engine = self.engine.lock().expect("engine lock");
            engine.handle_key(&mut self.composer, Key::from_x_state(keyval, state))
        };
        let after = self.composer.preedit();
        if !outcome.commit.is_empty() {
            Self::commit_text(&emitter, ibus_text(&outcome.commit, false)).await?;
        }
        if !outcome.commit.is_empty() || before != after {
            let cursor = after.chars().count() as u32;
            let visible = !after.is_empty();
            Self::update_preedit_text_with_mode(&emitter, ibus_text(&after, true), cursor, visible, PREEDIT_COMMIT)
                .await?;
        }
        Ok(outcome.consumed)
    }

    /// 앱이 조합 중 글자를 스스로 확정한 뒤 부른다(조합 모드 `PREEDIT_COMMIT`). 다시 확정하지 않는다.
    async fn reset(&mut self) {
        self.composer.flush();
    }

    async fn focus_in(&self) {}

    /// 포커스 대상이 사라질 때만 온다. 확정해도 받을 곳이 없으므로 비운다.
    async fn focus_out(&mut self) {
        self.composer.flush();
    }

    async fn set_capabilities(&self, _caps: u32) {}

    async fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    async fn set_cursor_location_relative(&self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    async fn set_surrounding_text(&self, _text: Variant<'_>, _cursor_pos: u32, _anchor_pos: u32) {}

    async fn destroy(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> fdo::Result<()> {
        if let Some(path) = header.path() {
            server.remove::<InputContext, _>(path).await?;
        }
        Ok(())
    }

    #[zbus(property)]
    async fn client_commit_preedit(&self) -> (bool,) {
        (self.client_commit_preedit,)
    }

    #[zbus(property)]
    async fn set_client_commit_preedit(&mut self, value: (bool,)) {
        self.client_commit_preedit = value.0;
    }

    #[zbus(property)]
    async fn content_type(&self) -> (u32, u32) {
        self.content_type
    }

    #[zbus(property)]
    async fn set_content_type(&mut self, value: (u32, u32)) {
        self.content_type = value;
    }

    #[zbus(signal)]
    async fn commit_text(emitter: &SignalEmitter<'_>, text: Variant<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text_with_mode(
        emitter: &SignalEmitter<'_>,
        text: Variant<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;
}

/// 끝난 앱의 입력 컨텍스트를 회수한다. 연결이 끊길 때까지 돈다.
async fn reap_vanished_clients(conn: &Connection, owners: Owners) -> zbus::Result<()> {
    let dbus = fdo::DBusProxy::new(conn).await?;
    let mut changes = dbus.receive_name_owner_changed().await?;
    while let Some(change) = changes.next().await {
        let args = change.args()?;
        if args.new_owner().is_some() {
            continue;
        }
        let paths = owners.lock().expect("owners lock").remove(args.name().as_str());
        for path in paths.into_iter().flatten() {
            // 앱이 이미 Destroy했으면 없는 객체다.
            let _ = conn.object_server().remove::<InputContext, _>(&path).await;
        }
    }
    Ok(())
}

/// 세션 버스에 포털 이름을 등록하고 요청을 처리한다. 세션 버스가 없거나 이름을 이미 누가 쓰면 실패한다.
pub async fn serve(engine: Arc<Mutex<Engine>>) -> zbus::Result<()> {
    let owners = Owners::default();
    let portal = Portal {
        engine,
        owners: owners.clone(),
        next_id: 0,
    };
    let conn = connection::Builder::session()
        .serve_at(IBUS_PATH, portal)
        .name(PORTAL_NAME)
        .build()
        .await?;
    log::info!("IBus 포털 시작: {PORTAL_NAME} (Qt: QT_IM_MODULE=ibus IBUS_USE_PORTAL=1)");
    reap_vanished_clients(&conn, owners).await
}
