use super::*;
use std::path::PathBuf;

/// The default mutator for `PathBuf` values.
///
/// See the [`path_buf()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct PathBufMutator {
    string_mutator: StringMutator<Char>,
}

/// Create a new mutator for `PathBuf` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::path::PathBuf;
///
/// let mut value = PathBuf::from("/home/user");
///
/// let mut mutator = m::path_buf();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value:?}");
/// }
///
/// // Example output:
/// //
/// //     value = "/home"
/// //     value = "/home.csv"
/// //     value = "/home.csv/opt"
/// //     value = "/home.csv"
/// //     value = "/home.avi"
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn path_buf() -> PathBufMutator {
    PathBufMutator::default()
}

impl Mutate<PathBuf> for PathBufMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut PathBuf) -> Result<()> {
        // Push a random path component.
        if !c.shrink() {
            c.mutation(|ctx| {
                match ctx.rng().gen_u8() % 27 {
                    0 => value.push("bin"),
                    1 => value.push("boot"),
                    2 => value.push("cdrom"),
                    3 => value.push("dev"),
                    4 => value.push("etc"),
                    5 => value.push("home"),
                    6 => value.push("lib"),
                    7 => value.push("lib32"),
                    8 => value.push("lib64"),
                    9 => value.push("lost+found"),
                    10 => value.push("media"),
                    11 => value.push("mnt"),
                    12 => value.push("opt"),
                    13 => value.push("proc"),
                    14 => value.push("root"),
                    15 => value.push("run"),
                    16 => value.push("sbin"),
                    17 => value.push("snap"),
                    18 => value.push("srv"),
                    19 => value.push("swapfile"),
                    20 => value.push("sys"),
                    21 => value.push("tmp"),
                    22 => value.push("usr"),
                    23 => value.push("var"),
                    24 => value.push(".."),
                    25 => value.push("."),
                    26 => value.push("/"),
                    _ => {
                        let component = self.string_mutator.generate(ctx)?;
                        value.push(&component)
                    }
                };
                Ok(())
            })?;
        }

        // Pop a component.
        if value.parent().is_some() {
            c.mutation(|_ctx| {
                value.pop();
                Ok(())
            })?;
        }

        // Set a random extension.
        c.mutation(|ctx| {
            match ctx.rng().gen_u8() % 35 {
                0 => value.set_extension("exe"),
                1 => value.set_extension("bin"),
                2 => value.set_extension("pdf"),
                3 => value.set_extension("doc"),
                4 => value.set_extension("docx"),
                5 => value.set_extension("txt"),
                6 => value.set_extension("csv"),
                7 => value.set_extension("odt"),
                8 => value.set_extension("jpg"),
                9 => value.set_extension("png"),
                10 => value.set_extension("gif"),
                11 => value.set_extension("bmp"),
                12 => value.set_extension("tif"),
                13 => value.set_extension("mp3"),
                14 => value.set_extension("wav"),
                15 => value.set_extension("aac"),
                16 => value.set_extension("flac"),
                17 => value.set_extension("mp4"),
                18 => value.set_extension("avi"),
                19 => value.set_extension("mov"),
                20 => value.set_extension("wmv"),
                21 => value.set_extension("zip"),
                22 => value.set_extension("rar"),
                23 => value.set_extension("tar"),
                24 => value.set_extension("gz"),
                25 => value.set_extension("rs"),
                26 => value.set_extension("js"),
                27 => value.set_extension("py"),
                28 => value.set_extension("rb"),
                29 => value.set_extension("c"),
                30 => value.set_extension("cc"),
                31 => value.set_extension("cpp"),
                32 => value.set_extension("h"),
                33 => value.set_extension("hh"),
                34 => value.set_extension("hpp"),
                _ => {
                    let ext = self.string_mutator.generate(ctx)?;
                    value.set_extension(&ext)
                }
            };
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<PathBuf> for PathBufMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<PathBuf> {
        let count = ctx.rng().inner().gen_range(1..=5);
        let mut path = PathBuf::new();
        for _ in 0..count {
            let component = self.string_mutator.generate(ctx)?;
            path.push(&component);
        }
        Ok(path)
    }
}

impl DefaultMutate for PathBuf {
    type DefaultMutate = PathBufMutator;
}
