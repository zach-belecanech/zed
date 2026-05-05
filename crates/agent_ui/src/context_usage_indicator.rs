use acp_thread::TokenUsage;
use gpui::{App, Hsla, Pixels, Window, px};
use ui::{CircularProgress, Color, Icon, IconName, IconSize, prelude::*};

const WARNING_PERCENTAGE: f64 = 75.0;
const CRITICAL_PERCENTAGE: f64 = 90.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextUsageLevel {
    Normal,
    Warning,
    Critical,
}

impl ContextUsageLevel {
    pub fn from_percentage(percentage: f64) -> Self {
        if percentage >= CRITICAL_PERCENTAGE {
            Self::Critical
        } else if percentage >= WARNING_PERCENTAGE {
            Self::Warning
        } else {
            Self::Normal
        }
    }

    pub fn from_tokens(used_tokens: u64, max_tokens: u64) -> Self {
        Self::from_percentage(usage_percentage(used_tokens, max_tokens))
    }

    pub fn warning_message(self) -> Option<&'static str> {
        match self {
            Self::Normal => None,
            Self::Warning | Self::Critical => Some("Quality may decline as limit nears."),
        }
    }

    pub fn ring_color(self, cx: &App) -> Hsla {
        match self {
            Self::Normal => cx.theme().colors().text_muted,
            Self::Warning => cx.theme().status().warning,
            Self::Critical => cx.theme().status().error,
        }
    }

    pub fn text_color(self) -> Color {
        match self {
            Self::Normal => Color::Muted,
            Self::Warning => Color::Warning,
            Self::Critical => Color::Error,
        }
    }
}

pub fn context_usage_percentage(usage: &TokenUsage) -> f64 {
    usage_percentage(usage.used_tokens, usage.max_tokens)
}

fn usage_percentage(used_tokens: u64, max_tokens: u64) -> f64 {
    if max_tokens == 0 {
        0.0
    } else {
        (used_tokens as f64 / max_tokens as f64) * 100.0
    }
}

#[derive(Clone, Copy, Debug)]
struct SplitTokenLimits {
    input_max_tokens: u64,
    output_max_tokens: u64,
}

#[derive(IntoElement)]
pub struct ContextUsageIndicator {
    usage: TokenUsage,
    size: Pixels,
    stroke_width: Pixels,
    split_limits: Option<SplitTokenLimits>,
}

impl ContextUsageIndicator {
    pub fn new(usage: TokenUsage) -> Self {
        Self {
            usage,
            size: px(16.0),
            stroke_width: px(2.0),
            split_limits: None,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn stroke_width(mut self, stroke_width: Pixels) -> Self {
        self.stroke_width = stroke_width;
        self
    }

    pub fn split_limits(mut self, input_max_tokens: u64, output_max_tokens: u64) -> Self {
        self.split_limits = Some(SplitTokenLimits {
            input_max_tokens,
            output_max_tokens,
        });
        self
    }
}

impl RenderOnce for ContextUsageIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(split_limits) = self.split_limits {
            h_flex()
                .gap_1p5()
                .child(render_segment(
                    IconName::ArrowUp,
                    self.usage.input_tokens,
                    split_limits.input_max_tokens,
                    self.size,
                    self.stroke_width,
                    cx,
                ))
                .child(render_segment(
                    IconName::ArrowDown,
                    self.usage.output_tokens,
                    split_limits.output_max_tokens,
                    self.size,
                    self.stroke_width,
                    cx,
                ))
                .into_any_element()
        } else {
            render_ring(
                self.usage.used_tokens,
                self.usage.max_tokens,
                self.size,
                self.stroke_width,
                cx,
            )
            .into_any_element()
        }
    }
}

fn render_segment(
    icon: IconName,
    used_tokens: u64,
    max_tokens: u64,
    size: Pixels,
    stroke_width: Pixels,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .gap_0p5()
        .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
        .child(render_ring(used_tokens, max_tokens, size, stroke_width, cx))
}

fn render_ring(
    used_tokens: u64,
    max_tokens: u64,
    size: Pixels,
    stroke_width: Pixels,
    cx: &App,
) -> impl IntoElement {
    let (value, max_value) = safe_progress_values(used_tokens, max_tokens);

    CircularProgress::new(value, max_value, size, cx)
        .stroke_width(stroke_width)
        .progress_color(ContextUsageLevel::from_tokens(used_tokens, max_tokens).ring_color(cx))
}

fn safe_progress_values(used_tokens: u64, max_tokens: u64) -> (f32, f32) {
    if max_tokens == 0 {
        (0.0, 1.0)
    } else {
        (used_tokens as f32, max_tokens as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(used_tokens: u64, max_tokens: u64) -> TokenUsage {
        TokenUsage {
            max_tokens,
            used_tokens,
            input_tokens: 0,
            output_tokens: 0,
            max_output_tokens: None,
        }
    }

    #[test]
    fn context_usage_thresholds_match_phase_three() {
        assert_eq!(ContextUsageLevel::from_percentage(74.9), ContextUsageLevel::Normal);
        assert_eq!(ContextUsageLevel::from_percentage(75.0), ContextUsageLevel::Warning);
        assert_eq!(ContextUsageLevel::from_percentage(89.9), ContextUsageLevel::Warning);
        assert_eq!(ContextUsageLevel::from_percentage(90.0), ContextUsageLevel::Critical);
    }

    #[test]
    fn context_usage_percentage_handles_zero_limit() {
        assert_eq!(context_usage_percentage(&usage(0, 0)), 0.0);
        assert_eq!(context_usage_percentage(&usage(50, 200)), 25.0);
    }
}