use chess_render::{ButtonIcon, ButtonSpec, ButtonType};

#[test]
fn display_type_selects_independent_optional_content() {
    let both = ButtonSpec {
        icon: Some(ButtonIcon::Reset),
        text: Some("RESET"),
        button_type: ButtonType::Text,
    };
    assert_eq!(both.displayed_icon(), None);
    assert_eq!(both.displayed_text(), Some("RESET"));
    let icon = ButtonSpec {
        button_type: ButtonType::Icon,
        ..both
    };
    assert_eq!(icon.displayed_icon(), Some(ButtonIcon::Reset));
    assert_eq!(icon.displayed_text(), None);
    let combined = ButtonSpec {
        button_type: ButtonType::IconAndText,
        ..both
    };
    assert_eq!(combined.displayed_icon(), both.icon);
    assert_eq!(combined.displayed_text(), both.text);
    for button_type in [ButtonType::Icon, ButtonType::Text, ButtonType::IconAndText] {
        let empty = ButtonSpec {
            icon: None,
            text: None,
            button_type,
        };
        assert_eq!(empty.displayed_icon(), None);
        assert_eq!(empty.displayed_text(), None);
        let text_only = ButtonSpec {
            icon: None,
            button_type,
            ..both
        };
        assert_eq!(text_only.displayed_icon(), None);
        assert_eq!(
            text_only.displayed_text(),
            if button_type == ButtonType::Icon {
                None
            } else {
                both.text
            }
        );
        let icon_only = ButtonSpec {
            text: None,
            button_type,
            ..icon
        };
        assert_eq!(icon_only.displayed_text(), None);
        assert_eq!(
            icon_only.displayed_icon(),
            if button_type == ButtonType::Text {
                None
            } else {
                both.icon
            }
        );
    }
}
