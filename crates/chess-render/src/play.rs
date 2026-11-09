//! Dedicated Play view. Existing product layout/settings and hit paths stay separate.
use crate::{DisplayMetrics, Gray8, HitTarget, Layout, LayoutError, Rect, RenderOutput};
use chess_core::{play::PlayApp, PieceKind};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayTarget {
    Square(usize),
    RatingDown,
    RatingUp,
    NewGame,
    Retry,
    Resign,
    Close,
    FullRefresh,
    Promotion(PieceKind),
    CancelPromotion,
}
#[derive(Clone, Copy, Debug)]
pub struct PlayLayout {
    pub base: Layout,
}
impl PlayLayout {
    pub fn new(metrics: DisplayMetrics) -> Result<Self, LayoutError> {
        let mut base = Layout::new(metrics)?;
        let touch = (metrics.dpi * crate::MIN_TOUCH_MM).div_ceil(254) * 10;
        base.toolbar.height = base.toolbar.height.max(touch);
        base.previous.y = base.toolbar.bottom() + 12;
        base.next.y = base.previous.y;
        base.status.y = base.previous.bottom() + 12;
        base.status.height = metrics.height.saturating_sub(base.status.y + 20);
        Ok(Self { base })
    }
    pub fn controls(self, app: &PlayApp) -> Vec<(PlayTarget, Rect, bool)> {
        let base = self.base;
        let gap = 12;
        let width = (base.toolbar.width - gap * 3) / 4;
        let mut controls = vec![
            (PlayTarget::Close, base.exit, true),
            (
                PlayTarget::RatingDown,
                Rect::new(
                    base.previous.x,
                    base.previous.y,
                    base.board_outer.width / 4,
                    base.previous.height,
                ),
                app.can_new() && app.rating > 600,
            ),
            (
                PlayTarget::RatingUp,
                Rect::new(
                    base.board_outer.right() - base.board_outer.width / 4,
                    base.next.y,
                    base.board_outer.width / 4,
                    base.next.height,
                ),
                app.can_new() && app.rating < 2600,
            ),
        ];
        for (index, (target, enabled)) in [
            (PlayTarget::NewGame, app.can_new()),
            (PlayTarget::Resign, app.can_move()),
            (PlayTarget::Retry, !app.in_flight()),
            (PlayTarget::FullRefresh, true),
        ]
        .into_iter()
        .enumerate()
        {
            controls.push((
                target,
                Rect::new(
                    base.toolbar.x + index as u32 * (width + gap),
                    base.toolbar.y,
                    width,
                    base.toolbar.height,
                ),
                enabled,
            ));
        }
        controls
    }
    pub fn hit_test(self, x: u32, y: u32, app: &PlayApp) -> Option<HitTarget> {
        let target = if self.base.exit.contains(x, y) {
            Some(PlayTarget::Close)
        } else if app.promotion.is_some() {
            self.base
                .promotion_choices
                .iter()
                .position(|r| r.contains(x, y))
                .map(|i| {
                    PlayTarget::Promotion(
                        [
                            PieceKind::Queen,
                            PieceKind::Rook,
                            PieceKind::Bishop,
                            PieceKind::Knight,
                        ][i],
                    )
                })
                .or_else(|| {
                    self.base
                        .promotion_cancel
                        .contains(x, y)
                        .then_some(PlayTarget::CancelPromotion)
                })
        } else {
            self.controls(app)
                .into_iter()
                .find(|(_, rect, enabled)| *enabled && rect.contains(x, y))
                .map(|c| c.0)
                .or_else(|| {
                    if !app.can_move() || !self.base.board.contains(x, y) {
                        return None;
                    }
                    let size = self.base.board.width / 8;
                    Some(PlayTarget::Square(
                        ((y - self.base.board.y) / size * 8 + (x - self.base.board.x) / size)
                            as usize,
                    ))
                })
        };
        target.map(HitTarget::Play)
    }
}
impl crate::ScreenState for PlayApp {
    fn board(&self) -> &chess_core::Board {
        &self.board
    }
    fn play(&self) -> Option<&PlayApp> {
        Some(self)
    }
}
pub fn render_play(app: &PlayApp, metrics: DisplayMetrics) -> Result<RenderOutput, LayoutError> {
    let play = PlayLayout::new(metrics)?;
    let base = play.base;
    let mut frame = Gray8::new(metrics.width, metrics.height, 255);
    let scale = (metrics.dpi / 100).clamp(2, 5);
    crate::font::draw_text_centered(
        &mut frame,
        Rect::new(
            base.header.x,
            base.header.y,
            base.exit.x.saturating_sub(base.header.x + 12),
            base.header.height,
        ),
        "Kindle Chess Play",
        scale,
        0,
    );
    eink_ui::board::draw_board(
        &mut frame,
        &app.board,
        false,
        eink_ui::board::BoardGeometry {
            board: base.board,
            board_outer: base.board_outer,
            gutter: base.coordinate_gutter(),
        },
        scale,
    );
    if let Some(square) = app.selected {
        let rect = base.square_rect(square);
        frame.stroke_rect(rect, 6, 0);
        frame.stroke_rect(rect.inset(7), 3, 255);
    }
    for (target, rect, enabled) in play.controls(app) {
        let label = match target {
            PlayTarget::Close => "CLOSE",
            PlayTarget::RatingDown => "-",
            PlayTarget::RatingUp => "+",
            PlayTarget::NewGame => "NEW GAME",
            PlayTarget::Retry => "RETRY",
            PlayTarget::Resign => "RESIGN",
            PlayTarget::FullRefresh => "REFRESH",
            _ => unreachable!(),
        };
        crate::render::draw_navigation_button(&mut frame, rect, label, enabled, scale);
    }
    crate::font::draw_text_centered(
        &mut frame,
        Rect::new(
            base.board_outer.x + base.board_outer.width / 4 + 12,
            base.previous.y,
            base.board_outer.width / 2 - 24,
            base.previous.height,
        ),
        &format!("Maia {}", app.rating),
        scale,
        0,
    );
    frame.stroke_rect(base.status, 2, 0);
    let last = app
        .snapshot
        .as_ref()
        .and_then(|s| s.last_move.as_deref())
        .unwrap_or("—");
    let text = format!("{}\nLast move: {last}", app.message);
    crate::font::draw_wrapped_text(&mut frame, base.status.inset(20), &text, scale, 0);
    if app.promotion.is_some() {
        frame.fill_rect(base.promotion_modal, 255);
        frame.stroke_rect(base.promotion_modal, 3, 0);
        for (index, label) in ["QUEEN", "ROOK", "BISHOP", "KNIGHT"]
            .into_iter()
            .enumerate()
        {
            crate::render::draw_navigation_button(
                &mut frame,
                base.promotion_choices[index],
                label,
                true,
                scale,
            );
        }
        crate::render::draw_navigation_button(
            &mut frame,
            base.promotion_cancel,
            "CANCEL",
            true,
            scale,
        );
    }
    Ok(RenderOutput {
        frame,
        layout: base,
        damage: vec![base.viewport],
        analysis: None,
        analysis_entry: None,
    })
}
