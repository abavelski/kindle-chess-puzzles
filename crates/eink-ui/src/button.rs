#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonType {
    Icon,
    Text,
    IconAndText,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonIcon {
    SendPosition,
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
