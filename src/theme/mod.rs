use {reclutch::display as gfx, thiserror::Error};

pub mod flat;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("failed to load theme resource: {0}")]
    ResourceError(#[from] reclutch::error::ResourceError),
    #[error("failed to load theme font: {0}")]
    FontError(#[from] reclutch::error::FontError),
}

pub struct Painter<O: 'static>(Option<Box<dyn AnyPainter>>, std::marker::PhantomData<O>);

pub trait TypedPainter: AnyPainter {
    type Object: 'static;

    fn paint(&mut self, obj: &mut Self::Object) -> Vec<gfx::DisplayCommand>;
    fn size_hint(&mut self, obj: &mut Self::Object) -> gfx::Size;
}

pub trait AnyPainter {
    fn paint(&mut self, obj: &mut dyn std::any::Any) -> Vec<gfx::DisplayCommand>;
    fn size_hint(&mut self, obj: &mut dyn std::any::Any) -> gfx::Size;
}

impl<P: TypedPainter> AnyPainter for P {
    #[inline]
    fn paint(&mut self, obj: &mut dyn std::any::Any) -> Vec<gfx::DisplayCommand> {
        TypedPainter::paint(self, obj.downcast_mut::<P::Object>().unwrap())
    }

    #[inline]
    fn size_hint(&mut self, obj: &mut dyn std::any::Any) -> gfx::Size {
        TypedPainter::size_hint(self, obj.downcast_mut::<P::Object>().unwrap())
    }
}

pub trait Theme {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter>;
    fn color(&self, c: &'static str) -> gfx::Color;
}

pub fn get_painter<O: 'static>(theme: &dyn Theme, p: &'static str) -> Painter<O> {
    Painter(Some(theme.painter(p)), Default::default())
}

pub fn paint<O: 'static>(
    obj: &mut O,
    p: impl Fn(&mut O) -> &mut Painter<O>,
) -> Vec<gfx::DisplayCommand> {
    let mut painter = p(obj).0.take().unwrap();
    let out = AnyPainter::paint(&mut *painter, obj);
    p(obj).0 = Some(painter);
    out
}

pub fn size_hint<O: 'static>(obj: &mut O, p: impl Fn(&mut O) -> &mut Painter<O>) -> gfx::Size {
    let mut painter = p(obj).0.take().unwrap();
    let out = AnyPainter::size_hint(&mut *painter, obj);
    p(obj).0 = Some(painter);
    out
}

pub mod painters {
    //! Standard painter definitions used by `kit`.
    //! For a theme to support `kit`, it must implement all of these.

    pub const BUTTON: &str = "button";
}

pub mod colors {
    //! Standard color definitions used by `kit`.
    //! For a theme to support `kit`, it must implement all of these.

    /// Color used by text and other foreground elements.
    pub const FOREGROUND: &str = "foreground";
    /// Color used to fill general background elements.
    pub const BACKGROUND: &str = "background";
    /// A less contrasting version of the foreground.
    pub const WEAK_FOREGROUND: &str = "weak_foreground";
    /// A less contrasting version of the background.
    pub const STRONG_FOREGROUND: &str = "strong_foreground";
}
