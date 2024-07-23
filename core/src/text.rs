//! Draw and interact with text.
pub mod editor;
pub mod highlighter;
pub mod paragraph;

pub use editor::Editor;
pub use highlighter::Highlighter;
pub use paragraph::Paragraph;

use crate::alignment;
use crate::{Border, Color, Pixels, Point, Rectangle, Size};

use std::borrow::Cow;
use std::hash::{Hash, Hasher};

/// A paragraph.
#[derive(Debug, Clone, Copy)]
pub struct Text<Content = String, Font = crate::Font> {
    /// The content of the paragraph.
    pub content: Content,

    /// The bounds of the paragraph.
    pub bounds: Size,

    /// The size of the [`Text`] in logical pixels.
    pub size: Pixels,

    /// The line height of the [`Text`].
    pub line_height: LineHeight,

    /// The font of the [`Text`].
    pub font: Font,

    /// The horizontal alignment of the [`Text`].
    pub horizontal_alignment: alignment::Horizontal,

    /// The vertical alignment of the [`Text`].
    pub vertical_alignment: alignment::Vertical,

    /// The [`Shaping`] strategy of the [`Text`].
    pub shaping: Shaping,
}

/// The shaping strategy of some text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Shaping {
    /// No shaping and no font fallback.
    ///
    /// This shaping strategy is very cheap, but it will not display complex
    /// scripts properly nor try to find missing glyphs in your system fonts.
    ///
    /// You should use this strategy when you have complete control of the text
    /// and the font you are displaying in your application.
    ///
    /// This is the default.
    #[default]
    Basic,
    /// Advanced text shaping and font fallback.
    ///
    /// You will need to enable this flag if the text contains a complex
    /// script, the font used needs it, and/or multiple fonts in your system
    /// may be needed to display all of the glyphs.
    ///
    /// Advanced shaping is expensive! You should only enable it when necessary.
    Advanced,
}

/// The height of a line of text in a paragraph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    /// A factor of the size of the text.
    Relative(f32),

    /// An absolute height in logical pixels.
    Absolute(Pixels),
}

impl LineHeight {
    /// Returns the [`LineHeight`] in absolute logical pixels.
    pub fn to_absolute(self, text_size: Pixels) -> Pixels {
        match self {
            Self::Relative(factor) => Pixels(factor * text_size.0),
            Self::Absolute(pixels) => pixels,
        }
    }
}

impl Default for LineHeight {
    fn default() -> Self {
        Self::Relative(1.3)
    }
}

impl From<f32> for LineHeight {
    fn from(factor: f32) -> Self {
        Self::Relative(factor)
    }
}

impl From<Pixels> for LineHeight {
    fn from(pixels: Pixels) -> Self {
        Self::Absolute(pixels)
    }
}

impl Hash for LineHeight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Relative(factor) => {
                state.write_u8(0);
                factor.to_bits().hash(state);
            }
            Self::Absolute(pixels) => {
                state.write_u8(1);
                f32::from(*pixels).to_bits().hash(state);
            }
        }
    }
}

/// The result of hit testing on text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hit {
    /// The point was within the bounds of the returned character index.
    CharOffset(usize),
}

impl Hit {
    /// Computes the cursor position of the [`Hit`] .
    pub fn cursor(self) -> usize {
        match self {
            Self::CharOffset(i) => i,
        }
    }
}

/// The difference detected in some text.
///
/// You will obtain a [`Difference`] when you [`compare`] a [`Paragraph`] with some
/// [`Text`].
///
/// [`compare`]: Paragraph::compare
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difference {
    /// No difference.
    ///
    /// The text can be reused as it is!
    None,

    /// A bounds difference.
    ///
    /// This normally means a relayout is necessary, but the shape of the text can
    /// be reused.
    Bounds,

    /// A shape difference.
    ///
    /// The contents, alignment, sizes, fonts, or any other essential attributes
    /// of the shape of the text have changed. A complete reshape and relayout of
    /// the text is necessary.
    Shape,
}

/// A renderer capable of measuring and drawing [`Text`].
pub trait Renderer: crate::Renderer {
    /// The font type used.
    type Font: Copy + PartialEq;

    /// The [`Paragraph`] of this [`Renderer`].
    type Paragraph: Paragraph<Font = Self::Font> + 'static;

    /// The [`Editor`] of this [`Renderer`].
    type Editor: Editor<Font = Self::Font> + 'static;

    /// The icon font of the backend.
    const ICON_FONT: Self::Font;

    /// The `char` representing a ✔ icon in the [`ICON_FONT`].
    ///
    /// [`ICON_FONT`]: Self::ICON_FONT
    const CHECKMARK_ICON: char;

    /// The `char` representing a ▼ icon in the built-in [`ICON_FONT`].
    ///
    /// [`ICON_FONT`]: Self::ICON_FONT
    const ARROW_DOWN_ICON: char;

    /// Returns the default [`Self::Font`].
    fn default_font(&self) -> Self::Font;

    /// Returns the default size of [`Text`].
    fn default_size(&self) -> Pixels;

    /// Draws the given [`Paragraph`] at the given position and with the given
    /// [`Color`].
    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );

    /// Draws the given [`Editor`] at the given position and with the given
    /// [`Color`].
    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );

    /// Draws the given [`Text`] at the given position and with the given
    /// [`Color`].
    fn fill_text(
        &mut self,
        text: Text<String, Self::Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );
}

/// A span of text.
#[derive(Debug, Clone)]
pub struct Span<'a, Link = (), Font = crate::Font> {
    /// The [`Fragment`] of text.
    pub text: Fragment<'a>,
    /// The size of the [`Span`] in [`Pixels`].
    pub size: Option<Pixels>,
    /// The [`LineHeight`] of the [`Span`].
    pub line_height: Option<LineHeight>,
    /// The font of the [`Span`].
    pub font: Option<Font>,
    /// The [`Color`] of the [`Span`].
    pub color: Option<Color>,
    /// The [`Background`] of the [`Span`].
    pub background: Option<Background>,
    /// The link of the [`Span`].
    pub link: Option<Link>,
}

impl<'a, Link, Font> Span<'a, Link, Font> {
    /// Creates a new [`Span`] of text with the given text fragment.
    pub fn new(fragment: impl IntoFragment<'a>) -> Self {
        Self {
            text: fragment.into_fragment(),
            size: None,
            line_height: None,
            font: None,
            color: None,
            background: None,
            link: None,
        }
    }

    /// Sets the size of the [`Span`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`Span`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = Some(line_height.into());
        self
    }

    /// Sets the font of the [`Span`].
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the font of the [`Span`], if any.
    pub fn font_maybe(mut self, font: Option<impl Into<Font>>) -> Self {
        self.font = font.map(Into::into);
        self
    }

    /// Sets the [`Color`] of the [`Span`].
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the [`Color`] of the [`Span`], if any.
    pub fn color_maybe(mut self, color: Option<impl Into<Color>>) -> Self {
        self.color = color.map(Into::into);
        self
    }

    /// Sets the [`Background`] of the [`Span`].
    pub fn background(mut self, background: impl Into<Background>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Sets the [`Background`] of the [`Span`], if any.
    pub fn background_maybe(
        mut self,
        background: Option<impl Into<Background>>,
    ) -> Self {
        self.background = background.map(Into::into);
        self
    }

    /// Sets the link of the [`Span`].
    pub fn link(mut self, link: impl Into<Link>) -> Self {
        self.link = Some(link.into());
        self
    }

    /// Sets the link of the [`Span`], if any.
    pub fn link_maybe(mut self, link: Option<impl Into<Link>>) -> Self {
        self.link = link.map(Into::into);
        self
    }

    /// Turns the [`Span`] into a static one.
    pub fn to_static(self) -> Span<'static, Link, Font> {
        Span {
            text: Cow::Owned(self.text.into_owned()),
            size: self.size,
            line_height: self.line_height,
            font: self.font,
            color: self.color,
            background: self.background,
            link: self.link,
        }
    }
}

impl<'a, Link, Font> From<&'a str> for Span<'a, Link, Font> {
    fn from(value: &'a str) -> Self {
        Span::new(value)
    }
}

impl<'a, Link, Font: PartialEq> PartialEq for Span<'a, Link, Font> {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.size == other.size
            && self.line_height == other.line_height
            && self.font == other.font
            && self.color == other.color
    }
}

/// A fragment of [`Text`].
///
/// This is just an alias to a string that may be either
/// borrowed or owned.
pub type Fragment<'a> = Cow<'a, str>;

/// A trait for converting a value to some text [`Fragment`].
pub trait IntoFragment<'a> {
    /// Converts the value to some text [`Fragment`].
    fn into_fragment(self) -> Fragment<'a>;
}

impl<'a> IntoFragment<'a> for Fragment<'a> {
    fn into_fragment(self) -> Fragment<'a> {
        self
    }
}

impl<'a, 'b> IntoFragment<'a> for &'a Fragment<'b> {
    fn into_fragment(self) -> Fragment<'a> {
        Fragment::Borrowed(self)
    }
}

impl<'a> IntoFragment<'a> for &'a str {
    fn into_fragment(self) -> Fragment<'a> {
        Fragment::Borrowed(self)
    }
}

impl<'a> IntoFragment<'a> for &'a String {
    fn into_fragment(self) -> Fragment<'a> {
        Fragment::Borrowed(self.as_str())
    }
}

impl<'a> IntoFragment<'a> for String {
    fn into_fragment(self) -> Fragment<'a> {
        Fragment::Owned(self)
    }
}

macro_rules! into_fragment {
    ($type:ty) => {
        impl<'a> IntoFragment<'a> for $type {
            fn into_fragment(self) -> Fragment<'a> {
                Fragment::Owned(self.to_string())
            }
        }

        impl<'a> IntoFragment<'a> for &$type {
            fn into_fragment(self) -> Fragment<'a> {
                Fragment::Owned(self.to_string())
            }
        }
    };
}

into_fragment!(char);
into_fragment!(bool);

into_fragment!(u8);
into_fragment!(u16);
into_fragment!(u32);
into_fragment!(u64);
into_fragment!(u128);
into_fragment!(usize);

into_fragment!(i8);
into_fragment!(i16);
into_fragment!(i32);
into_fragment!(i64);
into_fragment!(i128);
into_fragment!(isize);

into_fragment!(f32);
into_fragment!(f64);

/// The background style of text
#[derive(Debug, Clone, Copy)]
pub struct Background {
    /// The background [`Color`]
    pub color: Color,
    /// The background [`Border`]
    pub border: Border,
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background {
            color,
            border: Border::default(),
        }
    }
}
