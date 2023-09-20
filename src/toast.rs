use crossbeam_channel::{Receiver, Sender};
use egui::WidgetText;
use lazy_static::lazy_static;
use std::time::Duration;

lazy_static! {
    /// Toasts added since the last draw call. These are moved to the
    /// egui context's memory, so you are free to recreate the [`Toasts`] instance every frame.
    pub(crate) static ref TOASTS_CHANNEL: (Sender<Toast>, Receiver<Toast>) = {
        crossbeam_channel::unbounded()
    };
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ToastKind {
    Info,
    Warning,
    Error,
    Success,
    Custom(u32),
}

impl From<u32> for ToastKind {
    fn from(value: u32) -> ToastKind {
        ToastKind::Custom(value)
    }
}

#[derive(Clone)]
pub struct Toast {
    pub kind: ToastKind,
    pub text: WidgetText,
    pub options: ToastOptions,
}

impl Toast {
    /// Close the toast immediately
    pub fn close(&mut self) {
        self.options.ttl_sec = 0.0;
    }
    /// Push this toast to global toasts queue to show it.
    pub fn push(self) {
        let _ = TOASTS_CHANNEL.0.send(self);
    }
    /// Wrapper around `Toast{...}.push()`
    pub fn create(kind: ToastKind, text: impl Into<WidgetText>, options: ToastOptions) {
        Toast {
            kind: kind,
            text: text.into(),
            options: options,
        }
        .push()
    }
    /// Create default error toast
    pub fn error(text: impl Into<WidgetText>) {
        Toast::create(
            ToastKind::Error,
            text,
            ToastOptions::default().show_progress(false),
        )
    }
    /// Create default warning toast
    pub fn warning(text: impl Into<WidgetText>) {
        Toast::create(
            ToastKind::Warning,
            text,
            ToastOptions::default()
                .duration_in_seconds(5.0)
                .show_progress(true),
        )
    }
    /// Create default success toast
    pub fn success(text: impl Into<WidgetText>) {
        Toast::create(
            ToastKind::Success,
            text,
            ToastOptions::default()
                .duration_in_seconds(2.0)
                .show_progress(true),
        )
    }
    /// Create default info toast
    pub fn info(text: impl Into<WidgetText>) {
        Toast::create(
            ToastKind::Info,
            text,
            ToastOptions::default()
                .duration_in_seconds(2.0)
                .show_progress(true),
        )
    }
}

#[derive(Copy, Clone)]
pub struct ToastOptions {
    /// Whether the toast should include an icon.
    pub show_icon: bool,
    /// Whether the toast should visualize the remaining time
    pub show_progress: bool,
    /// The toast is removed when this reaches zero.
    pub(crate) ttl_sec: f64,
    /// Initial value of ttl_sec, used for progress
    pub(crate) initial_ttl_sec: f64,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            show_icon: true,
            show_progress: true,
            ttl_sec: f64::INFINITY,
            initial_ttl_sec: f64::INFINITY,
        }
    }
}

impl ToastOptions {
    /// Set duration of the toast. [None] duration means the toast never expires.
    pub fn duration(mut self, duration: impl Into<Option<Duration>>) -> Self {
        self.ttl_sec = duration
            .into()
            .map_or(f64::INFINITY, |duration| duration.as_secs_f64());
        self.initial_ttl_sec = self.ttl_sec;
        self
    }

    /// Set duration of the toast in milliseconds.
    pub fn duration_in_millis(self, millis: u64) -> Self {
        self.duration(Duration::from_millis(millis))
    }

    /// Set duration of the toast in seconds.
    pub fn duration_in_seconds(self, secs: f64) -> Self {
        self.duration(Duration::from_secs_f64(secs))
    }

    /// Visualize remaining time using a progress bar.
    pub fn show_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }

    /// Show type icon in the toast.
    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    /// Remaining time of the toast between 1..0
    pub fn progress(self) -> f64 {
        if self.ttl_sec.is_finite() && self.initial_ttl_sec > 0.0 {
            self.ttl_sec / self.initial_ttl_sec
        } else {
            0.0
        }
    }
}
