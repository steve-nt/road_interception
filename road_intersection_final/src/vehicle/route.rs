use sdl2::pixels::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    Straight,
    Left,
    Right,
}

impl Route {
    pub fn random() -> Self {
        static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let n = COUNTER.fetch_add(2, std::sync::atomic::Ordering::Relaxed) % 3;
        match n {
            0 => Route::Straight,
            1 => Route::Left,
            _ => Route::Right,
        }
    }

    /// Route colors for audit:
    /// - Straight → Blue
    /// - Left     → Yellow
    /// - Right    → Orange
    pub fn color(self) -> Color {
        match self {
            Route::Straight => Color::RGB(0, 80, 220),
            Route::Left => Color::RGB(230, 190, 0),
            Route::Right => Color::RGB(220, 100, 0),
        }
    }
}
