use std::borrow::Cow;

pub fn canonical<'a>(entry: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let entry: Cow<'a, str> = entry.into();
    let canonicaled = entry.as_ref().trim().to_lowercase();
    if entry.eq(&canonicaled) {
        return entry;
    } else {
        return Cow::Owned(canonicaled);
    }
}

pub fn trim<'a>(entry: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let entry: Cow<'a, str> = entry.into();
    let canonicaled = entry.as_ref().trim().to_string();
    if entry.eq(&canonicaled) {
        return entry;
    } else {
        return Cow::Owned(canonicaled);
    }
}
