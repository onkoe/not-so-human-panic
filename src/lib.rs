#![allow(clippy::needless_doctest_main)] // it's not needless ;)
//! Panic messages for humans
//!
//! Handles panics by calling
//! [`std::panic::set_hook`](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
//! to make errors nice for humans.
//!
//! ## Differences from Human-Panic
//!
//! While these two libraries are extremely similar, `not-so-human-panic` also tells your users why the program panicked. It also has some slight changes to wording, mostly to prod users to report whatever bug/problem they've found.
//!
//! That said, `human-panic` may look less scary. If your panics are due to "truly" unexpected values, or your users are less technical, `human-panic` may be the better option!
//!
//! ## Quick Start
//!
//! To begin, please add the library to your Rust project with `cargo add not-so-human-panic`. You can also edit your Cargo.toml manually!
//!
//! Afterwards, you should `use` the library and stick it somewhere nice. An entrypoint like `main` is probably a good place to start!
//!
//!
//! ```should_panic
//! use not_so_human_panic::setup_panic;
//!
//! fn main() {
//!    // If you want different info than your Cargo.toml provides,
//!    // please fill out a "Metadata" struct with your information
//!    setup_panic!();
//!
//!     // Fun little panic which will be happily explained
//!     panic!("oh hey im in the quick start :D");
//! }
//!
//! fn main_but_with_metadata() {
//!     // If you want different info than what's in your Cargo.toml,
//!     // fill out a "Metadata" struct with some different values
//!     setup_panic!(Metadata {
//!       name: env!("CARGO_PKG_NAME").into(),
//!        version: env!("CARGO_PKG_VERSION").into(),
//!        authors: "Some Guy <someone-somewhere@farts.com>".into(),
//!        homepage: "github.com/someone/some-crate".into(),
//!     });
//!
//!     panic!("ahhh you caught me!");
//! }
//!
//! ```
//!
//! ## Why?
//! When you're building a CLI, polish is super important. Even though Rust is
//! pretty great at safety, it's not unheard of to access the wrong index in a
//! vector or have an assert fail somewhere.
//!
//! When an error eventually occurs, you probably will want to know about it. So
//! instead of just providing an error message on the command line, we can create a
//! call to action for people to submit a report.
//!
//! This should empower people to engage in communication, lowering the chances
//! people might get frustrated. And making it easier to figure out what might be
//! causing bugs.
//!
//! ### Default Output
//!
//! ```txt
//! thread 'main' panicked at 'oops', examples/main.rs:2:3
//! note: Run with `RUST_BACKTRACE=1` for a backtrace.
//! ```
//!
//! ### Not-So-Human-Panic Output
//!
//! ```txt
//! Well, this is embarrassing.
//!
//! not-so-human-panic had a problem and crashed. It seems that the problem has to do with the following:
//! OMG EVERYTHING IS ON FIRE!!!
//!
//! If you'd like, you can help us diagnose the problem! Please feel free to send us a crash report using the instructions below.
//!
//! We have generated a report file at "/var/folders/zw/bpfvmq390lv2c6gn_6byyv0w0000gn/T/report-8351cad6-d2b5-4fe8-accd-1fcbf4538792.toml". Submit an issue or email with the subject of "single-panic-test Crash Report" and include the report as an attachment.
//!
//! - Authors: Yoshua Wuyts <yoshuawuyts@gmail.com>
//! - Homepage: https://github.com/onkoe/not-so-human-panic
//!
//! We take privacy very seriously - we don't perform any automated error collection. In order to improve the software, we rely on users like you to submit reports.
//!
//! Thank you kindly!
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]

pub mod report;
use report::{Method, Report};

use std::borrow::Cow;
use std::io::Result as IoResult;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};

/// A convenient metadata struct that describes a crate
///
/// See [`metadata!`]
pub struct Metadata {
    /// The crate version
    pub version: Cow<'static, str>,
    /// The crate name
    pub name: Cow<'static, str>,
    /// The list of authors of the crate
    pub authors: Cow<'static, str>,
    /// The URL of the crate's website
    pub homepage: Cow<'static, str>,
}

/// Initialize [`Metadata`]
#[macro_export]
macro_rules! metadata {
    () => {
        $crate::Metadata {
            version: env!("CARGO_PKG_VERSION").into(),
            name: env!("CARGO_PKG_NAME").into(),
            authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        }
    };
}

/// `not-so-human-panic` initialisation macro
///
/// You can either call this macro with no arguments `setup_panic!()` or
/// with a Metadata struct, if you don't want the error message to display
/// the values used in your `Cargo.toml` file.
///
/// The Metadata struct can't implement `Default` because of orphan rules, which
/// means you need to provide all fields for initialisation.
///
/// ```
/// use not_so_human_panic::setup_panic;
///
/// setup_panic!(Metadata {
///     name: env!("CARGO_PKG_NAME").into(),
///     version: env!("CARGO_PKG_VERSION").into(),
///     authors: "My Company Support <support@mycompany.com>".into(),
///     homepage: "support.mycompany.com".into(),
/// });
/// ```
#[macro_export]
macro_rules! setup_panic {
    ($meta:expr) => {
        #[allow(unused_imports)]
        use std::panic::{self, PanicInfo};
        #[allow(unused_imports)]
        use $crate::{handle_dump, print_msg, Metadata};

        match $crate::PanicStyle::default() {
            $crate::PanicStyle::Debug => {}
            $crate::PanicStyle::Human => {
                let meta = $meta;

                panic::set_hook(Box::new(move |info: &PanicInfo| {
                    let file_path = handle_dump(&meta, info.clone());
                    print_msg(file_path, &meta, info)
                        .expect("not-so-human-panic: printing error message to console failed");
                }));
            }
        }
    };

    () => {
        $crate::setup_panic!($crate::metadata!());
    };
}

/// Style of panic to be used
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PanicStyle {
    /// Normal panic
    Debug,
    /// Human-formatted panic
    Human,
}

impl Default for PanicStyle {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            PanicStyle::Debug
        } else {
            match ::std::env::var("RUST_BACKTRACE") {
                Ok(_) => PanicStyle::Debug,
                Err(_) => PanicStyle::Human,
            }
        }
    }
}

/// Utility function that prints a message to our human users
#[cfg(feature = "color")]
pub fn print_msg<P: AsRef<Path>>(
    file_path: Option<P>,
    meta: &Metadata,
    info: &PanicInfo,
) -> IoResult<()> {
    use std::io::Write as _;

    let stderr = anstream::stderr();
    let mut stderr = stderr.lock();

    write!(stderr, "{}", anstyle::AnsiColor::Red.render_fg())?;
    write_msg(&mut stderr, file_path, meta, info)?;
    write!(stderr, "{}", anstyle::Reset.render())?;

    Ok(())
}

#[cfg(not(feature = "color"))]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, meta: &Metadata) -> IoResult<()> {
    let stderr = std::io::stderr();
    let mut stderr = stderr.lock();

    write_msg(&mut stderr, file_path, meta)?;

    Ok(())
}

/// Prints our readable error message to the screen.
///
/// Utilizes `PanicInfo` to tell the user what went wrong.
fn write_msg<P: AsRef<Path>>(
    buffer: &mut impl std::io::Write,
    file_path: Option<P>,
    meta: &Metadata,
    panic_info: &PanicInfo,
) -> IoResult<()> {
    let (_version, name, authors, homepage) =
        (&meta.version, &meta.name, &meta.authors, &meta.homepage);

    let cause = get_panic_cause(panic_info);

    writeln!(buffer, "Well, this is embarrassing.\n")?;
    writeln!(
        buffer,
        "{name} had a problem and crashed. \
        It seems that the problem has to do with the following:"
    )?;

    writeln!(buffer, "{cause}\n")?;

    writeln!(
        buffer,
        "If you'd like, you can help us diagnose the \
    problem! Please feel free to send us a crash report using the instructions below.\n"
    )?;
    writeln!(
        buffer,
        "We have generated a report file at \"{}\". Submit an \
     issue or email with the subject of \"{} Crash Report\" and include the \
     report as an attachment.\n",
        match file_path {
            Some(fp) => format!("{}", fp.as_ref().display()),
            None => "<Failed to store file to disk>".to_string(),
        },
        name
    )?;

    if !homepage.is_empty() {
        writeln!(buffer, "- Homepage: {homepage}")?;
    }
    if !authors.is_empty() {
        writeln!(buffer, "- Authors: {authors}")?;
    }
    writeln!(
        buffer,
        "\nWe take privacy very seriously - we don't perform any \
     automated error collection. In order to improve the software, we rely on \
     users like you to submit reports.\n"
    )?;
    writeln!(buffer, "Thank you kindly!")?;

    Ok(())
}

/// Returns a panic cause, if a reason is given.
fn get_panic_cause(panic_info: &PanicInfo) -> String {
    #[cfg(feature = "nightly")]
    let message = panic_info.message().map(|m| format!("{}", m));

    #[cfg(not(feature = "nightly"))]
    let message = match (
        panic_info.payload().downcast_ref::<&str>(),
        panic_info.payload().downcast_ref::<String>(),
    ) {
        (Some(s), _) => Some(s.to_string()),
        (_, Some(s)) => Some(s.to_string()),
        (None, None) => None,
    };

    match message {
        Some(m) => m,
        None => "Unknown".into(),
    }

    // Note: The `None` case will almost NEVER happen. I couldn't find an immediately obvious way to make it occur.
    // Using unwrap(), panic!(), array[38924], etc. still provided some message.
    // I'd reckon we could twist a None out at some point, but I couldn't find anything common at all...
    //
    // Please let me know if you have some ideas which may neccessite other forms of handling.
}

/// Utility function handles dumping information to disk
pub fn handle_dump(meta: &Metadata, panic_info: &PanicInfo) -> Option<PathBuf> {
    let mut expl = String::new();

    let cause = get_panic_cause(panic_info);

    match panic_info.location() {
        Some(location) => expl.push_str(&format!(
            "Panic occurred in file '{}' at line {}\n",
            location.file(),
            location.line()
        )),
        None => expl.push_str("Panic location unknown.\n"),
    }

    let report = Report::new(&meta.name, &meta.version, Method::Panic, expl, cause);

    match report.persist() {
        Ok(f) => Some(f),
        Err(_) => {
            eprintln!("{}", report.serialize().unwrap());
            None
        }
    }
}
