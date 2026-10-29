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
/// display type implicitly; STANDARD deliberately keeps the existing text labels.
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

/// SMALL uses available icons while controls without artwork retain their text.
pub(crate) fn toolbar_button_for_size(target: HitTarget, small: bool) -> ButtonSpec<'static> {
    let mut button = toolbar_button(target);
    if small && button.icon.is_some() {
        button.button_type = ButtonType::Icon;
    }
    button
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_toolbar_uses_available_icons_and_keeps_note_text() {
        for target in [
            HitTarget::ToggleMode,
            HitTarget::ToggleOrientationLock,
            HitTarget::Reset,
            HitTarget::Flip,
        ] {
            assert_eq!(
                toolbar_button_for_size(target, false).button_type,
                ButtonType::Text
            );
            let small = toolbar_button_for_size(target, true);
            assert!(small.displayed_icon().is_some());
            assert_eq!(small.displayed_text(), None);
        }
        let note = toolbar_button_for_size(HitTarget::ToggleDescription, true);
        assert_eq!(note.displayed_text(), Some("NOTE"));
        assert_eq!(note.displayed_icon(), None);
    }
}
