//! Transient external-board UI protocol. No transport or durable preferences.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BoardSessionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Sending,
    Error(String),
}
impl BoardSessionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Disconnected => "DISCONNECTED",
            Self::Connecting => "CONNECTING",
            Self::Connected => "CONNECTED",
            Self::Sending => "SENDING",
            Self::Error(_) => "CONNECTION ERROR",
        }
    }
    pub fn idle(&self) -> bool {
        matches!(self, Self::Connected)
    }
    pub fn busy(&self) -> bool {
        matches!(self, Self::Sending)
    }
    pub fn connected(&self) -> bool {
        self.idle() || self.busy()
    }
    pub fn connection_control_label(&self) -> &'static str {
        if self.connected() {
            "DISCONNECT"
        } else {
            self.label()
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoardIntent {
    Connect,
    Disconnect,
    Send(String),
}
