#[macro_export]
macro_rules! non_empty_string {
    ($TypeName: ident) => {
        #[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $TypeName(String);

        impl $TypeName {
            pub fn inner(&self) -> String {
                self.0.clone()
            }
        }

        impl PartialEq<str> for $TypeName {
            fn eq(&self, other: &str) -> bool {
                &self.inner() == other
            }
        }

        impl std::fmt::Display for $TypeName {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<str> for $TypeName {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl TryFrom<String> for $TypeName {
            type Error = String;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.trim().is_empty() {
                    return Err("value cannot be empty".to_string());
                }
                Ok($TypeName(value))
            }
        }
    };
}
