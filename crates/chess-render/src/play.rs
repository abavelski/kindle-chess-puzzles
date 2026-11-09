//! Dedicated Play view. Existing product layout/settings and hit paths stay separate.
use crate::{DisplayMetrics, Gray8, HitTarget, Layout, LayoutError, Rect, RenderOutput};
use chess_core::{play::PlayApp, Color, PieceKind};
use eink_ui::board::oriented_square;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayTarget {
    Square(usize),
    RatingDown,
    RatingUp,
    ChooseColor(Color),
    NewGame,
    Retry,
    Resign,
    Close,
    FullRefresh,
    Promotion(PieceKind),
    CancelPromotion,
    HistoryPly(usize),
    HistoryPreviousPage(usize),
    HistoryNextPage(usize),
    Latest,
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
        let scale = (metrics.dpi / 100).clamp(2, 5);
        let message_height = eink_ui::font::text_line_height(scale) * 2 + 12;
        let reserved_history = (touch * 3 + message_height + 48).max(metrics.height / 4);
        let max_board = metrics.height.saturating_sub(
            base.board_outer.y + base.toolbar.height + base.previous.height + 56 + reserved_history,
        );
        base.limit_board_height(max_board)?;
        base.toolbar.y = base.board_outer.bottom() + 12;
        base.previous.y = base.toolbar.bottom() + 12;
        base.next.y = base.previous.y;
        base.status.y = base.previous.bottom() + 12;
        base.status.height = metrics.height.saturating_sub(base.status.y + 20);
        Ok(Self { base })
    }
    pub fn rating_rects(self) -> [Rect; 5] {
        let unit = (self.base.toolbar.width - 48) / 8;
        let mut x = self.base.toolbar.x;
        [1, 2, 1, 2, 2].map(|span| {
            let rect = Rect::new(
                x,
                self.base.previous.y,
                unit * span,
                self.base.previous.height,
            );
            x = rect.right() + 12;
            rect
        })
    }
    pub fn controls(self, app: &PlayApp) -> Vec<(PlayTarget, Rect, bool)> {
        let base = self.base;
        let gap = 12;
        let width = (base.toolbar.width - gap * 3) / 4;
        let row = self.rating_rects();
        let mut controls = vec![
            (PlayTarget::Close, base.exit, true),
            (
                PlayTarget::RatingDown,
                row[0],
                app.can_new() && app.rating > 600,
            ),
            (
                PlayTarget::RatingUp,
                row[2],
                app.can_new() && app.rating < 2600,
            ),
            (PlayTarget::ChooseColor(Color::White), row[3], app.can_new()),
            (PlayTarget::ChooseColor(Color::Black), row[4], app.can_new()),
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
    pub fn history(self, app: &PlayApp) -> PlayHistoryLayout {
        let panel = self.base.status.inset(12);
        let scale = (self.base.metrics.dpi / 100).clamp(2, 5);
        let touch = self.base.minimum_touch_px().max(120);
        let message = Rect::new(
            panel.x,
            panel.y,
            panel.width,
            eink_ui::font::text_line_height(scale) * 2 + 12,
        );
        let footer_y = panel.bottom().saturating_sub(touch);
        let body = Rect::new(
            panel.x,
            message.bottom() + 12,
            panel.width,
            footer_y.saturating_sub(message.bottom() + 24),
        );
        let rows = (body.height / touch).max(1) as usize;
        let capacity = rows * 2;
        let count = app.snapshot.as_ref().map_or(0, |s| s.ply);
        let requested = app
            .history_page
            .unwrap_or_else(|| app.view_ply.saturating_sub(1) / capacity);
        let window = eink_ui::move_list::PageWindow::new(count, capacity, requested);
        let mut chips = Vec::new();
        if let Some(snapshot) = &app.snapshot {
            let width = (body.width - 12) / 2;
            for (slot, index) in window.range.clone().enumerate() {
                let ply = index + 1;
                let prefix = if ply % 2 == 1 {
                    format!("{}.", ply.div_ceil(2))
                } else {
                    format!("{}...", ply / 2)
                };
                chips.push(PlayHistoryChip {
                    ply,
                    rect: Rect::new(
                        body.x + (slot as u32 % 2) * (width + 12),
                        body.y + (slot as u32 / 2) * touch,
                        width,
                        touch,
                    ),
                    label: format!("{prefix} {}", snapshot.move_label(ply).unwrap_or("")),
                });
            }
        }
        let width = (panel.width - 36) / 4;
        let footer = std::array::from_fn::<_, 4, _>(|i| {
            Rect::new(panel.x + i as u32 * (width + 12), footer_y, width, touch)
        });
        PlayHistoryLayout {
            window,
            chips,
            message,
            body,
            previous: footer[0],
            start: footer[1],
            latest: footer[2],
            next: footer[3],
        }
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
            self.history(app).hit_test(x, y, app).or_else(|| {
                self.controls(app)
                    .into_iter()
                    .find(|(_, rect, enabled)| *enabled && rect.contains(x, y))
                    .map(|c| c.0)
                    .or_else(|| {
                        if !app.can_move() || !self.base.board.contains(x, y) {
                            return None;
                        }
                        let size = self.base.board.width / 8;
                        Some(PlayTarget::Square(oriented_square(
                            ((y - self.base.board.y) / size * 8 + (x - self.base.board.x) / size)
                                as usize,
                            app.flipped(),
                        )))
                    })
            })
        };
        target.map(HitTarget::Play)
    }
}
impl crate::ScreenState for PlayApp {
    fn board(&self) -> &chess_core::Board {
        &self.board
    }
    fn flipped(&self) -> bool {
        PlayApp::flipped(self)
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
        app.flipped(),
        eink_ui::board::BoardGeometry {
            board: base.board,
            board_outer: base.board_outer,
            gutter: base.coordinate_gutter(),
        },
        scale,
    );
    if let Some(square) = app.selected {
        let rect = base.square_rect(oriented_square(square, app.flipped()));
        frame.stroke_rect(rect, 6, 0);
        frame.stroke_rect(rect.inset(7), 3, 255);
    }
    for (target, rect, enabled) in play.controls(app) {
        if target == PlayTarget::Close {
            crate::render::draw_close_button(
                &mut frame,
                crate::render::header_control_visual_rect(base, rect),
            );
            continue;
        }
        if let PlayTarget::ChooseColor(color) = target {
            crate::render::draw_button_spec(
                &mut frame,
                rect,
                crate::ButtonSpec {
                    icon: None,
                    text: Some(if color == Color::White {
                        "WHITE"
                    } else {
                        "BLACK"
                    }),
                    button_type: crate::ButtonType::Text,
                },
                app.player_color == color,
                enabled,
                scale,
            );
            continue;
        }
        let label = match target {
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
        play.rating_rects()[1],
        &format!("Maia {}", app.rating),
        scale,
        0,
    );
    frame.stroke_rect(base.status, 2, 0);
    let history = play.history(app);
    let message = if app.viewing_history() {
        format!(
            "{}\nViewing ply {} of {} — LATEST / PLAY",
            app.message,
            app.view_ply,
            app.snapshot.as_ref().map_or(0, |s| s.ply)
        )
    } else {
        app.message.clone()
    };
    crate::font::draw_wrapped_text(&mut frame, history.message, &message, scale, 0);
    if history.chips.is_empty() {
        crate::font::draw_text_centered(&mut frame, history.body, "No moves yet", scale, 128);
    }
    for chip in &history.chips {
        eink_ui::move_list::draw_move_chip(
            &mut frame,
            chip.rect,
            &chip.label,
            app.view_ply == chip.ply,
            scale,
        );
    }
    for (rect, label, enabled) in [
        (
            history.previous,
            format!(
                "< {}/{}",
                history.window.page + 1,
                history.window.page_count
            ),
            history.window.page > 0,
        ),
        (history.start, "START".into(), app.snapshot.is_some()),
        (
            history.latest,
            if app.viewing_history() {
                "LATEST / PLAY"
            } else {
                "LATEST"
            }
            .into(),
            app.snapshot.is_some(),
        ),
        (
            history.next,
            format!(
                "{}/{} >",
                history.window.page + 1,
                history.window.page_count
            ),
            history.window.page + 1 < history.window.page_count,
        ),
    ] {
        if enabled
            && !app.in_flight()
            && rect == history.start
            && app.viewing_history()
            && app.view_ply == 0
        {
            crate::render::draw_button_spec(
                &mut frame,
                rect,
                crate::ButtonSpec {
                    icon: None,
                    text: Some(&label),
                    button_type: crate::ButtonType::Text,
                },
                true,
                true,
                scale,
            );
        } else if enabled && !app.in_flight() {
            eink_ui::move_list::draw_page_control(&mut frame, rect, &label, scale);
        } else {
            crate::render::draw_navigation_button(&mut frame, rect, &label, false, scale);
        }
    }
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

#[derive(Clone, Debug)]
pub struct PlayHistoryChip {
    pub ply: usize,
    pub rect: Rect,
    pub label: String,
}
#[derive(Clone, Debug)]
pub struct PlayHistoryLayout {
    pub window: eink_ui::move_list::PageWindow,
    pub chips: Vec<PlayHistoryChip>,
    pub message: Rect,
    pub body: Rect,
    pub previous: Rect,
    pub start: Rect,
    pub latest: Rect,
    pub next: Rect,
}
impl PlayHistoryLayout {
    fn hit_test(&self, x: u32, y: u32, app: &PlayApp) -> Option<PlayTarget> {
        if app.in_flight() || app.snapshot.is_none() {
            return None;
        }
        for (rect, target, enabled) in [
            (
                self.previous,
                PlayTarget::HistoryPreviousPage(self.window.page.saturating_sub(1)),
                self.window.page > 0,
            ),
            (
                self.next,
                PlayTarget::HistoryNextPage(self.window.page + 1),
                self.window.page + 1 < self.window.page_count,
            ),
            (self.start, PlayTarget::HistoryPly(0), true),
            (self.latest, PlayTarget::Latest, true),
        ] {
            if enabled && rect.contains(x, y) {
                return Some(target);
            }
        }
        self.chips
            .iter()
            .find(|chip| chip.rect.contains(x, y))
            .map(|chip| PlayTarget::HistoryPly(chip.ply))
    }
}
