use nannou::color::*;

pub trait IntoNannouColor {
    fn into_rgb(&self) -> Rgb<u8>;
}

/// Extend colorous::Color with easy conversion to a nannou color.
impl IntoNannouColor for colorous::Color {
    fn into_rgb(&self) -> Rgb<u8> {
        Rgb::from(self.as_tuple())
    }
}
