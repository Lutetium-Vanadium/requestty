use std::fmt;

use ui::backend::Backend;

use super::Completions;
use crate::Answers;

macro_rules! handler {
    ($name:ident, $fn_trait:ident ( $($type:ty),* ) -> $return:ty) => {
        pub(super) enum $name<'a, T> {
            Sync(Box<dyn $fn_trait( $($type),* ) -> $return + 'a>),
            None,
        }

        impl<'a, T> $name<'a, T> {
            #[allow(unused)]
            pub(super) fn take(&mut self) -> Self {
                std::mem::replace(self, Self::None)
            }
        }

        impl<T> Default for $name<'_, T> {
            fn default() -> Self {
                Self::None
            }
        }

        impl<T: fmt::Debug> fmt::Debug for $name<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Sync(_) => f.write_str("Sync(_)"),
                    Self::None => f.write_str("None"),
                }
            }
        }
    };

    // The type signature of the function must only contain &T
    ($name:ident, ?Sized $fn_trait:ident ( $($type:ty),* ) -> $return:ty) => {
        pub(super) enum $name<'a, T: ?Sized> {
            Sync(Box<dyn $fn_trait( $($type),* ) -> $return + 'a>),
            None,
        }

        impl<'a, T: ?Sized> $name<'a, T> {
            #[allow(unused)]
            pub(super) fn take(&mut self) -> Self {
                std::mem::replace(self, Self::None)
            }
        }

        impl<T: ?Sized> Default for $name<'_, T> {
            fn default() -> Self {
                Self::None
            }
        }

        impl<T: fmt::Debug + ?Sized> fmt::Debug for $name<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Sync(_) => f.write_str("Sync(_)"),
                    Self::None => f.write_str("None"),
                }
            }
        }
    };
}

handler!(Filter, FnOnce(T, &Answers) -> T);
handler!(AutoComplete, FnMut(T, &Answers) -> Completions<T>);
handler!(Validate, ?Sized FnMut(&T, &Answers) -> Result<(), String>);
handler!(ValidateByVal, FnMut(T, &Answers) -> Result<(), String>);
handler!(ValidateOnKey, ?Sized FnMut(&T, &Answers) -> bool);
handler!(ValidateOnKeyByVal, FnMut(T, &Answers) -> bool);
handler!(Transform, ?Sized FnOnce(&T, &Answers, &mut dyn Backend) -> std::io::Result<()>);
handler!(
    TransformByVal,
    FnOnce(T, &Answers, &mut dyn Backend) -> std::io::Result<()>
);
