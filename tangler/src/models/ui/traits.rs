use std::fmt::Display;
use termcolor::ColorChoice;

pub(crate) trait ConsoleStyle: Display{
    fn determine_color_choice(&self, stream: atty::Stream) -> ColorChoice {
        if atty::is(stream) {
            if self.supports_truecolor() {
                ColorChoice::Always
            } else {
                ColorChoice::Auto
            }
        } else {
            ColorChoice::Never
        }
    }
    // Function to check if the terminal supports true color
    fn supports_truecolor(&self) -> bool {
        // This is a simple heuristic. Likely need a more robust check.
        std::env::var("COLORTERM").map_or(false, |colorterm| colorterm == "truecolor" || colorterm == "24bit")
    }
}