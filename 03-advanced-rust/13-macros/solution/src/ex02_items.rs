//! Exercise 2: Generating Items.

/// Counts token trees at compile time: `count!(a b c)` is `3usize`.
/// Recursive, so it's fine for small inputs (the recursion limit is 128).
#[macro_export]
macro_rules! count {
    () => { 0usize };
    ($head:tt $($tail:tt)*) => { 1usize + $crate::count!($($tail)*) };
}

/// A newtype around a numeric value. `$vis:vis` matches `pub`, `pub(crate)`
/// or nothing, so the caller decides visibility.
///
/// ```
/// m13_macros_solution::newtype!(pub Meters(f64));
/// let m: Meters = 3.5.into();
/// assert_eq!(m.value(), 3.5);
/// assert_eq!(m.to_string(), "3.5");
/// ```
#[macro_export]
macro_rules! newtype {
    ($(#[$meta:meta])* $vis:vis $name:ident($inner:ty)) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        $vis struct $name($inner);

        impl $name {
            pub fn value(self) -> $inner {
                self.0
            }
        }

        impl ::std::convert::From<$inner> for $name {
            fn from(v: $inner) -> Self {
                $name(v)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

/// An enum whose variants know their own names.
///
/// ```
/// m13_macros_solution::string_enum! {
///     pub enum Color { Red, Green, Blue }
/// }
/// assert_eq!(Color::Green.as_str(), "Green");
/// assert_eq!("Blue".parse::<Color>(), Ok(Color::Blue));
/// assert_eq!(Color::COUNT, 3);
/// ```
#[macro_export]
macro_rules! string_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name { $($variant),+ }

        impl $name {
            pub const ALL: [$name; $crate::count!($($variant)+)] = [$($name::$variant),+];
            pub const COUNT: usize = $crate::count!($($variant)+);

            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),+
                }
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::std::string::String;
            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    $(stringify!($variant) => ::std::result::Result::Ok($name::$variant),)+
                    other => ::std::result::Result::Err(::std::format!("unknown {}: {other}", stringify!($name))),
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

// Used by run() and by the tests.
crate::newtype!(pub Meters(f64));
crate::newtype!(
    /// Seconds, as a whole number.
    pub Seconds(u64)
);
crate::string_enum! {
    pub enum Color { Red, Green, Blue }
}

pub fn run() {
    let m: Meters = 3.5.into();
    println!("Meters: {m} (value {})", m.value());
    println!("Seconds: {}", Seconds::from(90));
    println!("Color::ALL = {:?}, COUNT = {}", Color::ALL, Color::COUNT);
    println!("\"Green\".parse() = {:?}", "Green".parse::<Color>());
    println!("\"Pink\".parse()  = {:?}", "Pink".parse::<Color>());
    println!("count!(a b c d) = {}", crate::count!(a b c d));
}
