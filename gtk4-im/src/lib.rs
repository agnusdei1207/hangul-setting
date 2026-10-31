//! GTK4 입력 모듈. GTK4는 XIM을 지원하지 않으므로 앱 안에 `hangul-setting` 엔진을 그대로 싣는다.
//!
//! GTK4가 `…/gtk-4.0/4.0.0/immodules/`의 GIO 모듈을 읽어 `gtk-im-module` 확장점의 `hangul-setting`을 찾는다.
//! 한/영 상태는 앱 프로세스 하나 안에서 공유한다.

use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};

use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, pango};
use gtk4 as gtk;
use hangul_setting::config;
use hangul_setting::engine::{Engine, Key};
use hangul_setting::hangul::Composer;

/// `GTK_IM_MODULE`에 쓰는 이름.
const MODULE_NAME: &str = "hangul-setting";

fn engine() -> &'static Mutex<Engine> {
    static ENGINE: OnceLock<Mutex<Engine>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let config = config::load().unwrap_or_else(|e| {
            glib::g_warning!(MODULE_NAME, "설정 오류, 기본 한/영 키를 씁니다: {e}");
            config::Config::default()
        });
        Mutex::new(Engine::new(config.toggle_keys))
    })
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct HangulContext {
        pub composer: RefCell<Composer>,
        /// 우리가 처리하지 않는 키를 받는 GTK 기본 입력기. GTK4 입력 칸은 일반 글자도 입력기의
        /// 확정으로만 넣으므로 영문·기호·데드키를 여기에 맡긴다.
        pub fallback: gtk::IMContextSimple,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HangulContext {
        const NAME: &'static str = "HangulSettingIMContext";
        type Type = super::HangulContext;
        type ParentType = gtk::IMContext;

        fn class_init(klass: &mut Self::Class) {
            // gtk4-rs 0.11의 get_preedit_string 연결 함수는 NULL 출력 인자를 확인하지 않는다. GTK 위젯은
            // 속성 목록 등에 NULL을 넘기므로 그대로 두면 앱이 죽는다. NULL을 건너뛰는 함수로 바꾼다.
            let klass = unsafe { &mut *(klass as *mut Self::Class).cast::<gtk::ffi::GtkIMContextClass>() };
            klass.get_preedit_string = Some(get_preedit_string);
        }
    }

    unsafe extern "C" fn get_preedit_string(
        ptr: *mut gtk::ffi::GtkIMContext,
        text_ptr: *mut *mut std::ffi::c_char,
        attrs_ptr: *mut *mut pango::ffi::PangoAttrList,
        cursor_ptr: *mut std::ffi::c_int,
    ) {
        use gtk::glib::translate::{FromGlibPtrBorrow, IntoGlibPtr};
        let obj: glib::translate::Borrowed<super::HangulContext> =
            unsafe { super::HangulContext::from_glib_borrow(ptr.cast()) };
        let (text, attrs, cursor) = obj.imp().preedit_string();
        unsafe {
            if !text_ptr.is_null() {
                *text_ptr = text.into_glib_ptr();
            }
            if !attrs_ptr.is_null() {
                *attrs_ptr = attrs.into_glib_ptr();
            }
            if !cursor_ptr.is_null() {
                *cursor_ptr = cursor;
            }
        }
    }

    impl ObjectImpl for HangulContext {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj().downgrade();
            self.fallback.connect_commit(move |_, text| {
                if let Some(obj) = obj.upgrade() {
                    obj.emit_by_name::<()>("commit", &[&text]);
                }
            });
        }
    }

    impl HangulContext {
        /// 조합기를 바꾼 뒤 확정과 조합 중 글자 변화를 알린다. 시그널 처리기가 `preedit_string`을
        /// 다시 부르므로 조합기를 빌린 채로 내보내지 않는다.
        fn announce(&self, before: &str, commit: &str) {
            let obj = self.obj();
            let after = self.composer.borrow().preedit();
            if !commit.is_empty() {
                obj.emit_by_name::<()>("commit", &[&commit]);
            }
            if before == after {
                return;
            }
            if before.is_empty() {
                obj.emit_by_name::<()>("preedit-start", &[]);
            }
            obj.emit_by_name::<()>("preedit-changed", &[]);
            if after.is_empty() {
                obj.emit_by_name::<()>("preedit-end", &[]);
            }
        }

        /// 조합 중 글자를 확정한다(포커스를 잃거나 앱이 초기화할 때).
        fn flush(&self) {
            let before = self.composer.borrow().preedit();
            let commit = self.composer.borrow_mut().flush();
            self.announce(&before, &commit);
        }
    }

    impl IMContextImpl for HangulContext {
        fn filter_keypress(&self, event: &gdk::Event) -> bool {
            let Some(key_event) = event.downcast_ref::<gdk::KeyEvent>() else {
                return false;
            };
            if event.event_type() != gdk::EventType::KeyPress {
                return self.fallback.filter_keypress(event);
            }
            let key = Key::from_x_state(key_event.keyval().into_glib(), key_event.modifier_state().bits());
            let before = self.composer.borrow().preedit();
            let outcome = {
                let mut composer = self.composer.borrow_mut();
                engine().lock().expect("engine lock").handle_key(&mut composer, key)
            };
            // 키를 넘기는 경우에도 확정을 먼저 내보내므로 앱은 조합 글자 다음에 그 키를 처리한다.
            self.announce(&before, &outcome.commit);
            outcome.consumed || self.fallback.filter_keypress(event)
        }

        fn preedit_string(&self) -> (glib::GString, pango::AttrList, i32) {
            let text = self.composer.borrow().preedit();
            let attrs = pango::AttrList::new();
            if !text.is_empty() {
                let mut underline = pango::AttrInt::new_underline(pango::Underline::Single);
                underline.set_start_index(0);
                underline.set_end_index(text.len() as u32);
                attrs.insert(underline);
            }
            let cursor = text.chars().count() as i32;
            (text.into(), attrs, cursor)
        }

        fn set_client_widget<P: IsA<gtk::Widget>>(&self, widget: Option<&P>) {
            self.fallback.set_client_widget(widget);
        }

        fn focus_in(&self) {
            self.fallback.focus_in();
        }

        fn focus_out(&self) {
            self.flush();
            self.fallback.focus_out();
        }

        fn reset(&self) {
            self.flush();
            self.fallback.reset();
        }
    }
}

glib::wrapper! {
    pub struct HangulContext(ObjectSubclass<imp::HangulContext>) @extends gtk::IMContext;
}

/// GIO가 모듈을 열 때 부른다. 정적으로 등록한 GType이 남아 있어야 하므로 모듈을 내려놓지 않게 한다.
///
/// # Safety
/// GIO가 유효한 `GIOModule`을 넘긴다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn g_io_module_load(module: *mut gio::ffi::GIOModule) {
    unsafe { glib::gobject_ffi::g_type_module_use(module.cast()) };
    // GTK는 앱이 이미 초기화했다. gtk4-rs는 Rust에서 `gtk::init`을 불렀는지 따로 기록하므로 알려 준다.
    // GTK는 초기화한 메인 스레드에서 입력 모듈을 읽는다.
    if unsafe { gtk::ffi::gtk_is_initialized() } != 0 && !gtk::is_initialized_main_thread() {
        unsafe { gtk::set_initialized() };
    }
    gio::IOExtensionPoint::implement("gtk-im-module", HangulContext::static_type(), MODULE_NAME, 10);
}

/// # Safety
/// GIO가 부른다. 모듈을 내려놓지 않으므로 할 일이 없다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn g_io_module_unload(_module: *mut gio::ffi::GIOModule) {}
