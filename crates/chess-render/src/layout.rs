//! DPI-aware deterministic layout and hit testing.

use crate::Rect;
use chess_core::{Action, PromotionChoice};

pub const MIN_TOUCH_MM: u32 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayMetrics {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutError {
    InvalidMetrics,
    TooSmall,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMetrics => formatter.write_str("display metrics must be non-zero"),
            Self::TooSmall => formatter.write_str("display is too small for the chess layout"),
        }
    }
}

impl std::error::Error for LayoutError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitTarget {
    Square(usize),
    ToggleMode,
    ToggleDescription,
    ToggleOrientationLock,
    Reset,
    Flip,
    Previous,
    Next,
    Promotion(PromotionChoice),
    CancelPromotion,
}

impl HitTarget {
    pub fn into_action(self) -> Action {
        match self {
            Self::Square(square) => Action::TapSquare(square),
            Self::ToggleMode => Action::ToggleMode,
            Self::ToggleDescription => Action::ToggleDescription,
            Self::ToggleOrientationLock => Action::ToggleOrientationLock,
            Self::Reset => Action::Reset,
            Self::Flip => Action::Flip,
            Self::Previous => Action::PreviousPuzzle,
            Self::Next => Action::NextPuzzle,
            Self::Promotion(choice) => Action::ChoosePromotion(choice),
            Self::CancelPromotion => Action::CancelPromotion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlTarget {
    pub rect: Rect,
    pub target: HitTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Layout {
    pub metrics: DisplayMetrics,
    pub viewport: Rect,
    pub header: Rect,
    pub board_outer: Rect,
    pub board: Rect,
    pub toolbar: Rect,
    pub toolbar_targets: [ControlTarget; 5],
    pub previous: Rect,
    pub next: Rect,
    pub status: Rect,
    pub promotion_modal: Rect,
    pub promotion_choices: [Rect; 4],
    pub promotion_cancel: Rect,
    square_size: u32,
    minimum_touch_px: u32,
    coordinate_gutter: u32,
}

impl Layout {
    pub fn new(metrics: DisplayMetrics) -> Result<Self, LayoutError> {
        if metrics.width == 0 || metrics.height == 0 || metrics.dpi == 0 {
            return Err(LayoutError::InvalidMetrics);
        }

        let margin = px_for_mm(metrics.dpi, 3).max(12);
        let gap = px_for_mm(metrics.dpi, 2).max(8);
        let small_gap = px_for_mm(metrics.dpi, 1).max(4);
        let coordinate_gutter = px_for_mm(metrics.dpi, 4).max(16);
        let header_height = px_for_mm(metrics.dpi, 10).max(48);
        let minimum_touch_px = px_for_mm(metrics.dpi, MIN_TOUCH_MM);
        let toolbar_height = minimum_touch_px;
        let nav_height = minimum_touch_px;
        let status_min = px_for_mm(metrics.dpi, 14).max(64);

        let horizontal_reserved = margin
            .saturating_mul(2)
            .saturating_add(coordinate_gutter.saturating_mul(2));
        let vertical_reserved = margin
            .saturating_mul(2)
            .saturating_add(header_height)
            .saturating_add(gap.saturating_mul(4))
            .saturating_add(coordinate_gutter.saturating_mul(2))
            .saturating_add(toolbar_height)
            .saturating_add(nav_height)
            .saturating_add(status_min);

        if metrics.width <= horizontal_reserved || metrics.height <= vertical_reserved {
            return Err(LayoutError::TooSmall);
        }

        let max_board_width = metrics.width - horizontal_reserved;
        let max_board_height = metrics.height - vertical_reserved;
        let board_size = max_board_width.min(max_board_height) / 8 * 8;
        if board_size < 8 * 24 {
            return Err(LayoutError::TooSmall);
        }
        let square_size = board_size / 8;

        let board_x = (metrics.width - board_size) / 2;
        let board_y = margin
            .saturating_add(header_height)
            .saturating_add(gap)
            .saturating_add(coordinate_gutter);
        let board = Rect::new(board_x, board_y, board_size, board_size);
        let board_outer = Rect::new(
            board_x.saturating_sub(coordinate_gutter),
            board_y.saturating_sub(coordinate_gutter),
            board_size.saturating_add(coordinate_gutter.saturating_mul(2)),
            board_size.saturating_add(coordinate_gutter.saturating_mul(2)),
        );
        let header = Rect::new(board_outer.x, margin, board_outer.width, header_height);

        let toolbar_y = board
            .bottom()
            .saturating_add(coordinate_gutter)
            .saturating_add(gap);
        let toolbar = Rect::new(board_outer.x, toolbar_y, board_outer.width, toolbar_height);

        let toolbar_targets = split_targets(
            toolbar,
            small_gap,
            [
                HitTarget::ToggleMode,
                HitTarget::ToggleDescription,
                HitTarget::ToggleOrientationLock,
                HitTarget::Reset,
                HitTarget::Flip,
            ],
        );
        if toolbar_targets
            .iter()
            .any(|target| target.rect.width < minimum_touch_px)
        {
            return Err(LayoutError::TooSmall);
        }

        let nav_y = toolbar.bottom().saturating_add(gap);
        let nav_total_gap = small_gap;
        let nav_width = board_outer.width.saturating_sub(nav_total_gap) / 2;
        let previous = Rect::new(board_outer.x, nav_y, nav_width, nav_height);
        let next = Rect::new(
            previous.right().saturating_add(nav_total_gap),
            nav_y,
            board_outer
                .right()
                .saturating_sub(previous.right().saturating_add(nav_total_gap)),
            nav_height,
        );
        if previous.width < minimum_touch_px || next.width < minimum_touch_px {
            return Err(LayoutError::TooSmall);
        }

        let status_y = previous.bottom().saturating_add(gap);
        let status_bottom = metrics.height.saturating_sub(margin);
        if status_bottom <= status_y {
            return Err(LayoutError::TooSmall);
        }
        let status = Rect::new(
            board_outer.x,
            status_y,
            board_outer.width,
            status_bottom - status_y,
        );

        let modal_width = (board_outer.width * 3 / 4)
            .max(minimum_touch_px.saturating_mul(5))
            .min(board_outer.width);
        let modal_height = minimum_touch_px
            .saturating_mul(3)
            .saturating_add(gap.saturating_mul(2));
        if modal_width > board_outer.width || modal_height > board_outer.height {
            return Err(LayoutError::TooSmall);
        }
        let promotion_modal = Rect::new(
            board_outer.x + (board_outer.width - modal_width) / 2,
            board_outer.y + (board_outer.height - modal_height) / 2,
            modal_width,
            modal_height,
        );

        let choice_y = promotion_modal.y + minimum_touch_px + gap;
        let choice_width = promotion_modal
            .width
            .saturating_sub(small_gap.saturating_mul(3))
            / 4;
        if choice_width < minimum_touch_px {
            return Err(LayoutError::TooSmall);
        }
        let promotion_choices = std::array::from_fn(|index| {
            let x = promotion_modal.x
                + u32::try_from(index).expect("choice index fits") * (choice_width + small_gap);
            Rect::new(x, choice_y, choice_width, minimum_touch_px)
        });
        let promotion_cancel = Rect::new(
            promotion_modal.x,
            choice_y
                .saturating_add(minimum_touch_px)
                .saturating_add(gap),
            promotion_modal.width,
            minimum_touch_px,
        );

        let layout = Self {
            metrics,
            viewport: Rect::new(0, 0, metrics.width, metrics.height),
            header,
            board_outer,
            board,
            toolbar,
            toolbar_targets,
            previous,
            next,
            status,
            promotion_modal,
            promotion_choices,
            promotion_cancel,
            square_size,
            minimum_touch_px,
            coordinate_gutter,
        };
        if !layout.viewport.contains_rect(layout.board_outer)
            || !layout.viewport.contains_rect(layout.header)
            || !layout.viewport.contains_rect(layout.toolbar)
            || !layout.viewport.contains_rect(layout.previous)
            || !layout.viewport.contains_rect(layout.next)
            || !layout.viewport.contains_rect(layout.status)
            || !layout.viewport.contains_rect(layout.promotion_modal)
        {
            return Err(LayoutError::TooSmall);
        }
        Ok(layout)
    }

    pub const fn square_size(&self) -> u32 {
        self.square_size
    }

    pub const fn minimum_touch_px(&self) -> u32 {
        self.minimum_touch_px
    }

    pub const fn coordinate_gutter(&self) -> u32 {
        self.coordinate_gutter
    }

    pub fn square_rect(&self, display_square: usize) -> Rect {
        debug_assert!(display_square < 64);
        let row = u32::try_from(display_square / 8).expect("board row fits");
        let column = u32::try_from(display_square % 8).expect("board column fits");
        Rect::new(
            self.board.x + column * self.square_size,
            self.board.y + row * self.square_size,
            self.square_size,
            self.square_size,
        )
    }

    pub fn hit_test(
        &self,
        x: u32,
        y: u32,
        flipped: bool,
        promotion_open: bool,
    ) -> Option<HitTarget> {
        if !self.viewport.contains(x, y) {
            return None;
        }

        if promotion_open {
            const CHOICES: [PromotionChoice; 4] = [
                PromotionChoice::Queen,
                PromotionChoice::Rook,
                PromotionChoice::Bishop,
                PromotionChoice::Knight,
            ];
            for (index, rect) in self.promotion_choices.iter().enumerate() {
                if rect.contains(x, y) {
                    return Some(HitTarget::Promotion(CHOICES[index]));
                }
            }
            if self.promotion_cancel.contains(x, y) {
                return Some(HitTarget::CancelPromotion);
            }
            return None;
        }

        if self.board.contains(x, y) {
            let column = (x - self.board.x) / self.square_size;
            let row = (y - self.board.y) / self.square_size;
            let display = usize::try_from(row * 8 + column).expect("display square fits");
            return Some(HitTarget::Square(if flipped {
                63 - display
            } else {
                display
            }));
        }

        for target in self.toolbar_targets {
            if target.rect.contains(x, y) {
                return Some(target.target);
            }
        }
        if self.previous.contains(x, y) {
            return Some(HitTarget::Previous);
        }
        if self.next.contains(x, y) {
            return Some(HitTarget::Next);
        }
        None
    }
}

fn split_targets<const N: usize>(
    rect: Rect,
    gap: u32,
    targets: [HitTarget; N],
) -> [ControlTarget; N] {
    let gap_total = gap.saturating_mul(u32::try_from(N.saturating_sub(1)).unwrap_or(u32::MAX));
    let available = rect.width.saturating_sub(gap_total);
    let width = available / u32::try_from(N).expect("target count fits");
    std::array::from_fn(|index| {
        let index_u32 = u32::try_from(index).expect("target index fits");
        let x = rect.x + index_u32 * (width + gap);
        let actual_width = if index + 1 == N {
            rect.right().saturating_sub(x)
        } else {
            width
        };
        ControlTarget {
            rect: Rect::new(x, rect.y, actual_width, rect.height),
            target: targets[index],
        }
    })
}

const fn px_for_mm(dpi: u32, millimeters: u32) -> u32 {
    dpi.saturating_mul(millimeters)
        .saturating_mul(10)
        .saturating_add(127)
        / 254
}
