use std::fmt;

use serde::{
    de::{Error, MapAccess, Unexpected, Visitor},
    Deserializer,
};

pub mod rune_list {
    use std::hint::unreachable_unchecked;

    use super::*;
    use crate::RuneList;

    pub fn deserialize<'de, D>(deserialiser: D) -> Result<RuneList, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RuneListVisitor;

        impl<'de> Visitor<'de> for RuneListVisitor {
            type Value = RuneList;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "the rune item definitions")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut rune_list = Vec::new();
                while let Some((rune_name, value)) =
                    map.next_entry::<Box<str>, u32>()?
                {
                    rune_list.push((rune_name, value));
                }

                rune_list.sort_unstable_by_key(|(_, value)| *value);

                if let Some(duplicated_value) =
                    rune_list.windows(2).find_map(|window| {
                        let &[(_, l_val), (_, r_val)] = window else {
                            unsafe { unreachable_unchecked() }
                        };
                        (l_val == r_val).then_some(l_val)
                    })
                {
                    return Err(A::Error::invalid_value(
                        Unexpected::Unsigned(duplicated_value as u64),
                        &"a value that hadn't already been used",
                    ));
                }

                Ok(rune_list.into_boxed_slice())
            }
        }

        deserialiser.deserialize_struct("runes", &[], RuneListVisitor)
    }
}
