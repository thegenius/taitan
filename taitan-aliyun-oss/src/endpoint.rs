use std::borrow::Cow;

pub struct Endpoint<'a> {
    region_id: Cow<'a, str>,
}

impl<'a> Endpoint<'a> {
    pub fn new(region_id: impl Into<Cow<'a, str>>) -> Self {
        Self {
            region_id: region_id.into(),
        }
    }

    pub fn value(&self) -> String {
        format!("oss-{}.aliyuncs.com", self.region_id)
    }

    pub fn of(region_id: impl Into<Cow<'a, str>>) -> String {
        format!("oss-{}.aliyuncs.com", region_id.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_endpoint() {
        let endpoint = Endpoint::of("us-west-1");
        assert_eq!(endpoint, "oss-us-west-1.aliyuncs.com");
    }
}
