// use super::utils::{canonical, trim};
// use std::borrow::Cow;

// pub struct Entry<'a> {
//     pub name: Cow<'a, str>,
//     pub value: Cow<'a, str>,
// }

// impl<'a> Entry<'a> {
//     pub fn new(name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Entry<'a> {
//         Self::with_value(name, value)
//     }

//     pub fn with_values<T>(name: impl Into<Cow<'a, str>>, values: impl AsRef<[T]>) -> Entry<'a>
//     where
//         T: AsRef<str>,
//     {
//         let value = values
//             .as_ref()
//             .iter()
//             .map(|e| e.as_ref().trim())
//             .collect::<Vec<&str>>()
//             .join(",");
//         Self {
//             name: canonical(name),
//             value: trim(value),
//         }
//     }

//     pub fn with_value(
//         name: impl Into<Cow<'a, str>>,
//         value: impl Into<Cow<'a, str>>,
//     ) -> HeaderEntry<'a> {
//         Self {
//             name: canonical(name),
//             value: trim(value),
//         }
//     }

//     pub fn format(&self) -> String {
//         format!("{}:{}\n", self.name, self.value)
//     }
// }
