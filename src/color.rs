use adw::{
    StyleManager,
    gtk::{
        gdk::RGBA,
        pango::{AttrColor, AttrList},
    },
};

#[derive(Clone)]
pub struct Color {
    color: String,
}

impl Color {
    fn new(color: &str) -> Self {
        let color = color.to_string();
        Color { color }
    }

    pub fn as_f64(&self) -> (f64, f64, f64) {
        let color = RGBA::parse(&self.color).unwrap();
        let red = color.red() as f64;
        let green = color.green() as f64;
        let blue = color.blue() as f64;

        (red, green, blue)
    }

    pub fn as_u16(&self) -> (u16, u16, u16) {
        let color = self.as_f64();
        let red = (color.0 * u16::MAX as f64) as u16;
        let green = (color.1 * u16::MAX as f64) as u16;
        let blue = (color.2 * u16::MAX as f64) as u16;

        (red, green, blue)
    }

    pub fn as_attrs(&self) -> AttrList {
        let color = self.as_u16();
        let attr = AttrColor::new_foreground(color.0, color.1, color.2);
        let attrs = AttrList::new();
        attrs.insert(attr);

        attrs
    }
}

#[allow(dead_code)]
pub struct ColorScheme {
    pub foreground: Color,
    pub foreground_dim: Color,
    pub blue: Color,
    pub red: Color,
}

impl ColorScheme {
    pub fn new() -> Self {
        let style_manager = StyleManager::default();
        if style_manager.is_dark() {
            ColorScheme {
                foreground: Color::new("#ffffff"),
                foreground_dim: Color::new("#999999"),
                blue: Color::new("#00efe0"),
                red: Color::new("#e51665"),
            }
        } else {
            ColorScheme {
                foreground: Color::new("#000000"),
                foreground_dim: Color::new("#666666"),
                blue: Color::new("#1c7a92"),
                red: Color::new("#e51665"),
            }
        }
    }

    pub fn change(&self, change: f64) -> Color {
        if change < 0.0 {
            self.red.clone()
        } else {
            self.blue.clone()
        }
    }
}
