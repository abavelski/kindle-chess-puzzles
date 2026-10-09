//! Button content is independent of its presentation policy.

use crate::HitTarget;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonType {
    Icon,
    Text,
    IconAndText,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonIcon {
    Analysis,
    FreeBoard,
    Lock,
    Reset,
    Flip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ButtonSpec<'a> {
    pub icon: Option<ButtonIcon>,
    pub text: Option<&'a str>,
    pub button_type: ButtonType,
}

impl<'a> ButtonSpec<'a> {
    pub fn displayed_icon(self) -> Option<ButtonIcon> {
        match self.button_type {
            ButtonType::Icon | ButtonType::IconAndText => self.icon,
            ButtonType::Text => None,
        }
    }

    pub fn displayed_text(self) -> Option<&'a str> {
        match self.button_type {
            ButtonType::Text | ButtonType::IconAndText => self.text,
            ButtonType::Icon => None,
        }
    }
}

/// Shared by toolbar layout and rendering. Adding content never changes the
/// display type implicitly; the current toolbar deliberately keeps text labels.
pub(crate) fn toolbar_button(target: HitTarget) -> ButtonSpec<'static> {
    let (icon, text, button_type) = match target {
        HitTarget::ToggleAnalysis => (Some(ButtonIcon::Analysis), None, ButtonType::Icon),
        HitTarget::ToggleMode => (Some(ButtonIcon::FreeBoard), Some("FREE"), ButtonType::Text),
        HitTarget::ToggleDescription => (None, Some("NOTE"), ButtonType::Text),
        HitTarget::ToggleOrientationLock => {
            (Some(ButtonIcon::Lock), Some("LOCK"), ButtonType::Text)
        }
        HitTarget::Reset => (Some(ButtonIcon::Reset), Some("RESET"), ButtonType::Text),
        HitTarget::Flip => (Some(ButtonIcon::Flip), Some("FLIP"), ButtonType::Text),
        _ => unreachable!("not a toolbar button"),
    };
    ButtonSpec {
        icon,
        text,
        button_type,
    }
}
