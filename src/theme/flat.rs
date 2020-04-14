use {super::*, reclutch::display as gfx};

pub struct FlatTheme;

impl Theme for FlatTheme {
    fn painter(&self, p: &'static str) -> Box<dyn AnyPainter> {
        match p {
            _ => unimplemented!(),
        }
    }

    fn color(&self, c: &'static str) -> gfx::Color {
        match c {
            _ => unimplemented!(),
        }
    }
}
