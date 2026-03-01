use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Base colors
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub title: Color,
    
    // Interactive elements
    pub highlight: Color,
    pub highlight_bg: Color,
    pub selected: Color,
    pub selected_bg: Color,
    
    // Status and info
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub status_bar_key: Color,
    pub help_bar_bg: Color,
    pub help_bar_fg: Color,
    
    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // Additional
    pub gauge: Color,
    pub secondary: Color,
}

impl ColorPalette {
    pub fn midnight_commander() -> Self {
        Self {
            background: Color::Blue,
            foreground: Color::White,
            border: Color::Cyan,
            title: Color::Yellow,
            
            highlight: Color::Black,
            highlight_bg: Color::Cyan,
            selected: Color::Yellow,
            selected_bg: Color::DarkGray,
            
            status_bar_bg: Color::Black,
            status_bar_fg: Color::White,
            status_bar_key: Color::Cyan,
            help_bar_bg: Color::Black,
            help_bar_fg: Color::Gray,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::LightRed,
            info: Color::Cyan,
            
            gauge: Color::Cyan,
            secondary: Color::Gray,
        }
    }
    
    pub fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::White,
            border: Color::White,
            title: Color::Cyan,
            
            highlight: Color::White,
            highlight_bg: Color::Blue,
            selected: Color::Cyan,
            selected_bg: Color::DarkGray,
            
            status_bar_bg: Color::DarkGray,
            status_bar_fg: Color::White,
            status_bar_key: Color::Cyan,
            help_bar_bg: Color::DarkGray,
            help_bar_fg: Color::White,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
            
            gauge: Color::Cyan,
            secondary: Color::Gray,
        }
    }
    
    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Gray,
            border: Color::DarkGray,
            title: Color::White,
            
            highlight: Color::White,
            highlight_bg: Color::Rgb(40, 40, 40),
            selected: Color::White,
            selected_bg: Color::Rgb(60, 60, 60),
            
            status_bar_bg: Color::Rgb(30, 30, 30),
            status_bar_fg: Color::Gray,
            status_bar_key: Color::White,
            help_bar_bg: Color::Rgb(30, 30, 30),
            help_bar_fg: Color::Gray,
            
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            
            gauge: Color::Blue,
            secondary: Color::DarkGray,
        }
    }
    
    pub fn minimal() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::White,
            border: Color::White,
            title: Color::White,
            
            highlight: Color::Black,
            highlight_bg: Color::White,
            selected: Color::Black,
            selected_bg: Color::Gray,
            
            status_bar_bg: Color::White,
            status_bar_fg: Color::Black,
            status_bar_key: Color::Black,
            help_bar_bg: Color::White,
            help_bar_fg: Color::Black,
            
            success: Color::White,
            warning: Color::White,
            error: Color::White,
            info: Color::White,
            
            gauge: Color::White,
            secondary: Color::Gray,
        }
    }
    
    pub fn monokai() -> Self {
        Self {
            background: Color::Rgb(39, 40, 34),
            foreground: Color::Rgb(248, 248, 242),
            border: Color::Rgb(117, 113, 94),
            title: Color::Rgb(166, 226, 46),
            
            highlight: Color::Rgb(39, 40, 34),
            highlight_bg: Color::Rgb(117, 113, 94),
            selected: Color::Rgb(253, 151, 31),
            selected_bg: Color::Rgb(73, 72, 62),
            
            status_bar_bg: Color::Rgb(39, 40, 34),
            status_bar_fg: Color::Rgb(248, 248, 242),
            status_bar_key: Color::Rgb(102, 217, 239),
            help_bar_bg: Color::Rgb(39, 40, 34),
            help_bar_fg: Color::Rgb(117, 113, 94),
            
            success: Color::Rgb(166, 226, 46),
            warning: Color::Rgb(253, 151, 31),
            error: Color::Rgb(249, 38, 114),
            info: Color::Rgb(102, 217, 239),
            
            gauge: Color::Rgb(102, 217, 239),
            secondary: Color::Rgb(117, 113, 94),
        }
    }
    
    pub fn solarized_dark() -> Self {
        Self {
            background: Color::Rgb(0, 43, 54),
            foreground: Color::Rgb(131, 148, 150),
            border: Color::Rgb(88, 110, 117),
            title: Color::Rgb(181, 137, 0),
            
            highlight: Color::Rgb(0, 43, 54),
            highlight_bg: Color::Rgb(88, 110, 117),
            selected: Color::Rgb(181, 137, 0),
            selected_bg: Color::Rgb(7, 54, 66),
            
            status_bar_bg: Color::Rgb(7, 54, 66),
            status_bar_fg: Color::Rgb(131, 148, 150),
            status_bar_key: Color::Rgb(38, 139, 210),
            help_bar_bg: Color::Rgb(7, 54, 66),
            help_bar_fg: Color::Rgb(88, 110, 117),
            
            success: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            info: Color::Rgb(38, 139, 210),
            
            gauge: Color::Rgb(42, 161, 152),
            secondary: Color::Rgb(88, 110, 117),
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::default()
    }
}
