/// Helper macro for defining capabilities
#[macro_export]
macro_rules! define_capabilities {
    (
        $(
            $(#[$meta:meta])*
            $name:ident = $value:expr;
        )*
    ) => {
        /// String constant for the capability
        $(
            $(#[$meta])*
            pub const $name: &str = $value;
        )*

        /// Type-safe identifier for capabilities
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "specta", derive(specta::Type))]
        #[allow(non_camel_case_types)]
        pub enum CapabilityId {
            $(
                $name,
            )*
        }

        impl CapabilityId {
            /// Returns the Capabilities value, e.g. `REQUIRES_API_KEY` -> `vmm.mod.requires_api_key`
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        CapabilityId::$name => $value,
                    )*
                }
            }
        }
    };
}
