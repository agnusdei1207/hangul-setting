//! 조합 중 글자 창. xterm처럼 조합 중 글자 그리기를 입력기에 맡기는 XIM 앱(over-the-spot)을 위해
//! 커서 자리에 작은 창을 띄우고 나눔고딕으로 그린다.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ConfigureWindowAux, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask, Gcontext, ImageFormat, ImageOrder,
    StackMode, Window, WindowClass,
};

/// `fonts-nanum` 패키지의 나눔고딕. 패키지가 이 폰트에 의존한다.
pub const FONT_PATH: &str = "/usr/share/fonts/truetype/nanum/NanumGothic.ttf";
const FONT_SIZE: f32 = 18.0;
const PADDING: usize = 3;

/// 회색조 그림(0 = 검정, 255 = 흰색).
pub struct Picture {
    pub width: usize,
    pub height: usize,
    /// 위에서 기준선까지 픽셀 수.
    pub baseline: usize,
    pub pixels: Vec<u8>,
}

/// 글자를 흰 바탕에 검게 그리고 밑줄을 긋는다.
pub fn render(font: &fontdue::Font, text: &str) -> Picture {
    let line = font.horizontal_line_metrics(FONT_SIZE).expect("가로쓰기 폰트");
    let ascent = line.ascent.ceil() as usize;
    let height = ascent + (-line.descent).ceil() as usize + 2 * PADDING;
    let baseline = PADDING + ascent;
    let glyphs: Vec<_> = text.chars().map(|c| font.rasterize(c, FONT_SIZE)).collect();
    let advance: f32 = glyphs.iter().map(|(m, _)| m.advance_width).sum();
    let width = advance.ceil() as usize + 2 * PADDING;

    let mut pixels = vec![255u8; width * height];
    let mut pen = PADDING as f32;
    for (metrics, coverage) in &glyphs {
        let left = pen.round() as i32 + metrics.xmin;
        let top = baseline as i32 - metrics.height as i32 - metrics.ymin;
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let (x, y) = (left + col as i32, top + row as i32);
                if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
                    let pixel = &mut pixels[y as usize * width + x as usize];
                    *pixel = (*pixel).min(255 - coverage[row * metrics.width + col]);
                }
            }
        }
        pen += metrics.advance_width;
    }
    let underline = (baseline + 2).min(height - 1);
    pixels[underline * width + PADDING..underline * width + width - PADDING].fill(0);
    Picture {
        width,
        height,
        baseline,
        pixels,
    }
}

pub struct PreeditWindow<'c, C: Connection> {
    conn: &'c C,
    screen_num: usize,
    window: Window,
    gc: Gcontext,
    depth: u8,
    lsb_first: bool,
    font: fontdue::Font,
    /// 다시 그리기(Expose)용 마지막 그림.
    shown: Option<Picture>,
}

impl<'c, C: Connection> PreeditWindow<'c, C> {
    /// 폰트가 없거나 화면이 32비트 픽셀이 아니면 `None`이다. 이때 조합 중 글자는 보이지 않을 뿐 입력은 된다.
    pub fn new(conn: &'c C, screen_num: usize) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let font = match std::fs::read(FONT_PATH) {
            Ok(bytes) => fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())?,
            Err(e) => {
                log::warn!("조합 창 꺼짐: {FONT_PATH}: {e}");
                return Ok(None);
            }
        };
        let setup = conn.setup();
        let screen = &setup.roots[screen_num];
        let depth = screen.root_depth;
        if !setup
            .pixmap_formats
            .iter()
            .any(|f| f.depth == depth && f.bits_per_pixel == 32)
        {
            log::warn!("조합 창 꺼짐: 깊이 {depth} 화면이 32비트 픽셀이 아님");
            return Ok(None);
        }
        let window = conn.generate_id()?;
        let aux = CreateWindowAux::new()
            .override_redirect(1)
            .background_pixel(screen.white_pixel)
            .border_pixel(screen.black_pixel)
            .event_mask(EventMask::EXPOSURE);
        conn.create_window(
            depth,
            window,
            screen.root,
            0,
            0,
            1,
            1,
            1,
            WindowClass::INPUT_OUTPUT,
            0,
            &aux,
        )?;
        x11rb::wrapper::ConnectionExt::change_property8(
            conn,
            x11rb::protocol::xproto::PropMode::REPLACE,
            window,
            x11rb::protocol::xproto::AtomEnum::WM_NAME,
            x11rb::protocol::xproto::AtomEnum::STRING,
            b"hangul-setting-preedit",
        )?;
        let gc = conn.generate_id()?;
        conn.create_gc(gc, window, &CreateGCAux::new())?;
        let lsb_first = setup.image_byte_order == ImageOrder::LSB_FIRST;
        Ok(Some(PreeditWindow {
            conn,
            screen_num,
            window,
            gc,
            depth,
            lsb_first,
            font,
            shown: None,
        }))
    }

    pub fn window(&self) -> Window {
        self.window
    }

    /// `(x, y)`는 앱 창 `relative_to` 기준 커서 기준선 위치다. 빈 글자면 창을 숨긴다.
    pub fn show(&mut self, text: &str, relative_to: Window, x: i16, y: i16) -> Result<(), Box<dyn std::error::Error>> {
        if text.is_empty() {
            self.shown = None;
            self.conn.unmap_window(self.window)?;
            self.conn.flush()?;
            return Ok(());
        }
        let root = self.conn.setup().roots[self.screen_num].root;
        let spot = self.conn.translate_coordinates(relative_to, root, x, y)?.reply()?;
        let (x, y) = (spot.dst_x, spot.dst_y);
        let picture = render(&self.font, text);
        let top = i32::from(y) - picture.baseline as i32;
        let aux = ConfigureWindowAux::new()
            .x(i32::from(x))
            .y(top)
            .width(picture.width as u32)
            .height(picture.height as u32)
            .stack_mode(StackMode::ABOVE);
        self.conn.configure_window(self.window, &aux)?;
        self.conn.map_window(self.window)?;
        self.shown = Some(picture);
        self.redraw()
    }

    /// 창이 다시 보이게 될 때(Expose) 부른다.
    pub fn redraw(&self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(picture) = &self.shown else { return Ok(()) };
        let mut data = Vec::with_capacity(picture.pixels.len() * 4);
        for &v in &picture.pixels {
            let pixel = u32::from(v) * 0x010101;
            data.extend_from_slice(&if self.lsb_first {
                pixel.to_le_bytes()
            } else {
                pixel.to_be_bytes()
            });
        }
        let (w, h) = (picture.width as u16, picture.height as u16);
        self.conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gc,
            w,
            h,
            0,
            0,
            0,
            self.depth,
            &data,
        )?;
        self.conn.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 나눔고딕이 없는 개발 PC에서는 건너뛴다. CI(`CI` 환경 변수)에서는 글꼴이 없으면 실패한다.
    fn font() -> Option<fontdue::Font> {
        match std::fs::read(FONT_PATH) {
            Ok(bytes) => Some(fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).expect("나눔고딕")),
            Err(e) if std::env::var_os("CI").is_some() => panic!("CI에 {FONT_PATH}가 없음: {e}"),
            Err(_) => None,
        }
    }

    #[test]
    fn render_draws_ink_and_grows_with_text() {
        let Some(font) = font() else { return };
        let one = render(&font, "한");
        let two = render(&font, "한글");
        assert!(two.width > one.width);
        assert_eq!(one.pixels.len(), one.width * one.height);
        let ink = one.pixels.iter().filter(|&&v| v < 128).count();
        assert!(ink > 20, "글자 픽셀 {ink}");
    }
}
