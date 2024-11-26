/// Contains bindings to imageboard's native [`Archive`].
///
/// [`Archive`]: crate::models::archive::Archive
pub mod archive;
///
/// Contains bidnings to list of [`Board`]s and their attributes.
///
/// [`Board`]: crate::models::board::Board
pub mod board;
/// Contains bindings for imageboard's native [`Catalog`]
///
/// [`Catalog`]: crate::models::catalog::Catalog
pub mod catalog;
/// Contains bindings for specific [`Thread`]s
///
/// [`Thread`]: crate::models::thread::Thread
pub mod thread;
/// Contains bindings for threads + their attributes in a [`ThreadList`]
///
/// [`ThreadList`]: crate::models::threadlist::ThreadList
pub mod threadlist;

#[derive(Debug, Clone, Default)]
pub(crate) struct Metadata {
    url: String,
    pub(crate) last_modified: String,
}

impl Metadata {
    pub(crate) fn url(&self) -> &str {
        &self.url
    }
}

pub(crate) fn maybe_de_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <Option<u32> as serde::Deserialize>::deserialize(deserializer)?;
    // If it's 1, return Some(true), if it's 0, return Some(false), else None
    Ok(value.map(|v| v == 1))
}

pub(crate) fn de_bool<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: u32 = serde::Deserialize::deserialize(d)?;
    Ok(value == 1)
}

pub(crate) mod macros {
    macro_rules! str_opt_ref {
        ($x:expr) => {
            $x.as_ref().map(|x| x.as_ref())
        };
    }

    pub(crate) use str_opt_ref;
}
