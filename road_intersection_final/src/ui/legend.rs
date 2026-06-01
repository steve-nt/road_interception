use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::vehicle::route::Route;

const PANEL_W: u32 = 118;
const PANEL_H: u32 = 88;
const PANEL_MARGIN: i32 = 10;
const PANEL_PAD: i32 = 8;
const ROW_H: i32 = 22;
const SWATCH: u32 = 12;
const CHAR_W: i32 = 6;
const GLYPH_H: i32 = 7;

pub fn draw(canvas: &mut WindowCanvas) {
    let panel_x = WINDOW_WIDTH as i32 - PANEL_W as i32 - PANEL_MARGIN;
    let panel_y = WINDOW_HEIGHT as i32 - PANEL_H as i32 - PANEL_MARGIN;

    canvas.set_draw_color(Color::RGB(18, 28, 18));
    canvas
        .fill_rect(Rect::new(panel_x, panel_y, PANEL_W, PANEL_H))
        .unwrap();

    canvas.set_draw_color(Color::RGB(200, 200, 200));
    canvas
        .draw_rect(Rect::new(panel_x, panel_y, PANEL_W, PANEL_H))
        .unwrap();

    let title_y = panel_y + PANEL_PAD;
    draw_text(
        canvas,
        panel_x + PANEL_PAD,
        title_y,
        "Routes",
        Color::RGB(220, 220, 220),
    );

    let entries = [
        (Route::Left, "Left"),
        (Route::Straight, "Straight"),
        (Route::Right, "Right"),
    ];

    let rows_top = panel_y + PANEL_PAD + GLYPH_H + 6;
    let text_x = panel_x + PANEL_PAD + SWATCH as i32 + 6;
    let max_text_x = panel_x + PANEL_W as i32 - PANEL_PAD;

    for (i, (route, label)) in entries.iter().enumerate() {
        let row_y = rows_top + i as i32 * ROW_H;
        let swatch_x = panel_x + PANEL_PAD;
        let swatch_y = row_y + (ROW_H - SWATCH as i32) / 2;
        let text_y = row_y + (ROW_H - GLYPH_H) / 2;

        canvas.set_draw_color(route.color());
        canvas
            .fill_rect(Rect::new(swatch_x, swatch_y, SWATCH, SWATCH))
            .unwrap();
        canvas.set_draw_color(Color::RGB(30, 30, 30));
        canvas
            .draw_rect(Rect::new(swatch_x, swatch_y, SWATCH, SWATCH))
            .unwrap();

        draw_text_clipped(
            canvas,
            text_x,
            text_y,
            max_text_x,
            label,
            Color::RGB(235, 235, 235),
        );
    }
}

fn draw_text(canvas: &mut WindowCanvas, x: i32, y: i32, text: &str, color: Color) {
    draw_text_clipped(canvas, x, y, i32::MAX, text, color);
}

fn draw_text_clipped(
    canvas: &mut WindowCanvas,
    x: i32,
    y: i32,
    max_x: i32,
    text: &str,
    color: Color,
) {
    let mut cx = x;
    for ch in text.chars() {
        if cx + 5 > max_x {
            break;
        }
        draw_char(canvas, cx, y, ch, color);
        cx += CHAR_W;
    }
}

fn draw_char(canvas: &mut WindowCanvas, x: i32, y: i32, ch: char, color: Color) {
    let Some(rows) = glyph(ch) else {
        return;
    };
    canvas.set_draw_color(color);
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                canvas
                    .fill_rect(Rect::new(x + col, y + row as i32, 1, 1))
                    .unwrap();
            }
        }
    }
}

/// 5×7 pixel glyphs for legend labels.
fn glyph(ch: char) -> Option<[u8; 7]> {
    Some(match ch {
        'A' | 'a' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'E' | 'e' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'f' => [0x1E, 0x10, 0x10, 0x1C, 0x10, 0x10, 0x10],
        'g' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x0E],
        'h' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'i' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'L' | 'l' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'n' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'o' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'R' | 'r' => [0x1E, 0x11, 0x11, 0x1E, 0x12, 0x11, 0x11],
        'S' | 's' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E],
        't' => [0x1C, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'u' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        _ => return None,
    })
}
