//! XIM 요청을 엔진에 잇는다. 입력 컨텍스트마다 조합기를 하나 둔다.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hangul_setting::engine::{Engine, Key};
use hangul_setting::hangul::Composer;
use hangul_setting::keymap::Keymap;

use crate::preedit_window::PreeditWindow;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::KeyPressEvent;
use xim::{
    CommitData, ForwardEventFlag, InputContext, InputStyle, Request, Server, ServerCore, ServerError, ServerHandler,
    UserInputContext,
};

pub const KEY_PRESS_MASK: u32 = 1;

/// 앱 하나의 입력 컨텍스트. XIM 연결마다 통신 창이 따로 있고 id는 연결 안에서만 유일하다.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct IcAddress {
    com_win: u32,
    input_method_id: u16,
    input_context_id: u16,
}

/// 앱에 되돌려 줄 키 하나.
struct HeldKey {
    client_win: u32,
    request: Request,
}

pub struct Handler<'c, C: Connection> {
    /// 한/영 모드는 IBus 프론트엔드와 함께 쓴다.
    engine: Arc<Mutex<Engine>>,
    keymap: Keymap,
    /// 지금 처리 중인 XIM 메시지가 도착한 통신 창. 메인 루프가 `filter_event` 전에 정한다.
    pub com_win: u32,
    /// 확정과 함께 앱에 넘길 키. 앱이 확정을 처리했다는 SyncReply가 오면 보낸다.
    ///
    /// libX11은 "IM을 이미 거친 키" 표시를 한 비트로만 관리한다. 확정용 가짜 키와 되돌린 키가
    /// 한꺼번에 쌓이면 GTK3에서 둘 중 하나가 표시를 잃고 순서가 뒤집혀 스페이스·엔터가 조합 중이던
    /// 글자보다 먼저 들어간다. 확정을 동기로 보내면 앱은 가짜 키를 처리한 뒤에 SyncReply를 보낸다.
    held: HashMap<IcAddress, HeldKey>,
    /// 조합 중 글자를 스스로 그리지 않는 앱(over-the-spot, root)을 위한 창. 폰트가 없으면 `None`.
    preedit_window: Option<PreeditWindow<'c, C>>,
}

impl<'c, C: Connection> Handler<'c, C> {
    pub fn new(engine: Arc<Mutex<Engine>>, keymap: Keymap, preedit_window: Option<PreeditWindow<'c, C>>) -> Self {
        Handler {
            engine,
            keymap,
            com_win: 0,
            held: HashMap::new(),
            preedit_window,
        }
    }

    /// 조합 창이 다시 보이게 됐을 때.
    pub fn handle_expose(&self, window: u32) {
        if let Some(preedit) = self.preedit_window.as_ref().filter(|p| p.window() == window)
            && let Err(e) = preedit.redraw()
        {
            log::warn!("조합 창 다시 그리기 실패: {e}");
        }
    }

    /// 앱이 콜백을 받으면 앱이 그리고, 아니면 커서 자리의 조합 창에 그린다.
    fn draw_preedit<S: Server>(
        &mut self,
        server: &mut S,
        ic: &mut InputContext,
        text: &str,
    ) -> Result<(), ServerError> {
        if ic.input_style().contains(InputStyle::PREEDIT_CALLBACKS) {
            return server.preedit_draw(ic, text);
        }
        let Some(window) = &mut self.preedit_window else {
            return Ok(());
        };
        let Some(app_win) = ic.app_focus_win().or(ic.app_win()) else {
            return Ok(());
        };
        let spot = ic.preedit_spot();
        window
            .show(text, app_win.get(), spot.x, spot.y)
            .map_err(|e| ServerError::Internal(e.to_string()))
    }

    /// 조합 중 글자를 비우고 `commit`을 확정한 뒤 새 조합 중 글자를 그린다.
    fn show<S: Server>(
        &mut self,
        server: &mut S,
        ic: &mut InputContext,
        commit: &str,
        preedit: &str,
    ) -> Result<(), ServerError> {
        if !commit.is_empty() {
            self.draw_preedit(server, ic, "")?;
            server.commit(ic, commit)?;
        }
        self.draw_preedit(server, ic, preedit)
    }

    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.keymap = keymap;
    }

    fn address(&self, ic: &InputContext) -> IcAddress {
        IcAddress {
            com_win: self.com_win,
            input_method_id: ic.input_method_id().get(),
            input_context_id: ic.input_context_id().get(),
        }
    }

    /// SyncReply를 받으면 그 컨텍스트에 붙잡아 둔 키를 보낸다.
    pub fn handle_sync_reply<S: ServerCore>(
        &mut self,
        server: &mut S,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ServerError> {
        let address = IcAddress {
            com_win: self.com_win,
            input_method_id,
            input_context_id,
        };
        self.release(server, address)
    }

    fn release<S: ServerCore>(&mut self, server: &mut S, address: IcAddress) -> Result<(), ServerError> {
        match self.held.remove(&address) {
            Some(key) => server.send_req(key.client_win, key.request),
            None => Ok(()),
        }
    }
}

fn forward_request(ic: &InputContext, xev: &KeyPressEvent) -> Request {
    Request::ForwardEvent {
        input_method_id: ic.input_method_id().get(),
        input_context_id: ic.input_context_id().get(),
        serial_number: 0,
        flag: ForwardEventFlag::empty(),
        xev: xim::XEvent {
            response_type: xev.response_type,
            detail: xev.detail,
            sequence: xev.sequence,
            time: xev.time,
            root: xev.root,
            event: xev.event,
            child: xev.child,
            root_x: xev.root_x,
            root_y: xev.root_y,
            event_x: xev.event_x,
            event_y: xev.event_y,
            state: u16::from(xev.state),
            same_screen: xev.same_screen,
        },
    }
}

fn sync_commit_request(ic: &InputContext, text: &str) -> Request {
    Request::Commit {
        input_method_id: ic.input_method_id().get(),
        input_context_id: ic.input_context_id().get(),
        data: CommitData::Chars {
            commited: xim_ctext::utf8_to_compound_text(text),
            syncronous: true,
        },
    }
}

impl<C: Connection, S: Server<XEvent = KeyPressEvent> + ServerCore> ServerHandler<S> for Handler<'_, C> {
    type InputStyleArray = [InputStyle; 4];
    type InputContextData = Composer;

    fn new_ic_data(&mut self, _server: &mut S, _style: InputStyle) -> Result<Composer, ServerError> {
        Ok(Composer::default())
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [
            InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_NOTHING | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NONE,
        ]
    }

    fn filter_events(&self) -> u32 {
        KEY_PRESS_MASK
    }

    fn handle_connect(&mut self, _server: &mut S) -> Result<(), ServerError> {
        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut S,
        user_ic: &mut UserInputContext<Composer>,
    ) -> Result<(), ServerError> {
        server.set_event_mask(&user_ic.ic, KEY_PRESS_MASK, 0)
    }

    fn handle_destroy_ic(
        &mut self,
        server: &mut S,
        mut user_ic: UserInputContext<Composer>,
    ) -> Result<(), ServerError> {
        let address = self.address(&user_ic.ic);
        self.held.remove(&address);
        self.draw_preedit(server, &mut user_ic.ic, "")
    }

    fn handle_reset_ic(
        &mut self,
        server: &mut S,
        user_ic: &mut UserInputContext<Composer>,
    ) -> Result<String, ServerError> {
        self.draw_preedit(server, &mut user_ic.ic, "")?;
        Ok(user_ic.user_data.flush())
    }

    fn handle_set_focus(
        &mut self,
        _server: &mut S,
        _user_ic: &mut UserInputContext<Composer>,
    ) -> Result<(), ServerError> {
        Ok(())
    }

    fn handle_unset_focus(
        &mut self,
        server: &mut S,
        user_ic: &mut UserInputContext<Composer>,
    ) -> Result<(), ServerError> {
        let address = self.address(&user_ic.ic);
        self.release(server, address)?;
        let commit = user_ic.user_data.flush();
        self.show(server, &mut user_ic.ic, &commit, "")
    }

    fn handle_set_ic_values(
        &mut self,
        _server: &mut S,
        _user_ic: &mut UserInputContext<Composer>,
    ) -> Result<(), ServerError> {
        Ok(())
    }

    fn handle_forward_event(
        &mut self,
        server: &mut S,
        user_ic: &mut UserInputContext<Composer>,
        xev: &KeyPressEvent,
    ) -> Result<bool, ServerError> {
        // SyncReply보다 다음 키가 먼저 오면 붙잡아 둔 키를 먼저 보내 순서를 지킨다.
        let address = self.address(&user_ic.ic);
        self.release(server, address)?;

        let key = Key::from_x_state(self.keymap.base_keysym(xev.detail), u32::from(u16::from(xev.state)));
        let before = user_ic.user_data.preedit();
        let outcome = self
            .engine
            .lock()
            .expect("engine lock")
            .handle_key(&mut user_ic.user_data, key);
        let after = user_ic.user_data.preedit();

        if !outcome.consumed && !outcome.commit.is_empty() {
            self.draw_preedit(server, &mut user_ic.ic, "")?;
            server.send_req(
                user_ic.ic.client_win(),
                sync_commit_request(&user_ic.ic, &outcome.commit),
            )?;
            let held = HeldKey {
                client_win: user_ic.ic.client_win(),
                request: forward_request(&user_ic.ic, xev),
            };
            self.held.insert(address, held);
            return Ok(true);
        }
        if !outcome.commit.is_empty() || before != after {
            self.show(server, &mut user_ic.ic, &outcome.commit, &after)?;
        }
        Ok(outcome.consumed)
    }
}
