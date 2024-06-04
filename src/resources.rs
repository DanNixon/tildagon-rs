// https://github.com/esp-rs/esp-hal/blob/1597443bf1a87cb80160cf279fe21ef3a2205796/esp-hal/src/macros.rs#L371
macro_rules! assign_resources {
    {
        $(#[$struct_meta:meta])*
        $vis:vis $struct_name:ident<$struct_lt:lifetime> {
            $(
                $(#[$group_meta:meta])*
                $group_name:ident : $group_struct:ident<$group_lt:lifetime> {
                    $(
                        $(#[$resource_meta:meta])*
                        $resource_name:ident : $resource_field:ident
                    ),*
                    $(,)?
                }
            ),+
            $(,)?
        }
    } => {
        // Group structs
        $(
            $(#[$group_meta])*
            #[allow(missing_docs)]
            $vis struct $group_struct<$group_lt> {
                $(
                    $(#[$resource_meta])*
                    pub $resource_name: esp_hal::peripherals::$resource_field<$group_lt>,
                )+
            }

            impl<$group_lt> $group_struct<$group_lt> {
                /// Unsafely create an instance of the assigned peripherals out of thin air.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of the contained peripherals at a time.
                pub unsafe fn steal() -> Self {
                    unsafe {
                        Self {
                            $($resource_name: esp_hal::peripherals::$resource_field::steal()),*
                        }
                    }
                }

                /// Creates a new reference to the peripheral group with a shorter lifetime.
                ///
                /// Use this method if you would like to keep working with the peripherals after
                /// you dropped the drivers that consume this.
                pub fn reborrow(&mut self) -> $group_struct<'_> {
                    $group_struct {
                        $($resource_name: self.$resource_name.reborrow()),*
                    }
                }
            }
        )+

        // Outer struct
        $(#[$struct_meta])*
        /// Assigned resources.
        $vis struct $struct_name<$struct_lt> {
            $( pub $group_name: $group_struct<$struct_lt>, )+
        }

        impl<$struct_lt> $struct_name<$struct_lt> {
            /// Unsafely create an instance of the assigned peripherals out of thin air.
            ///
            /// # Safety
            ///
            /// You must ensure that you're only using one instance of the contained peripherals at a time.
            pub unsafe fn steal() -> Self {
                unsafe {
                    Self {
                        $($group_name: $group_struct::steal()),*
                    }
                }
            }

            /// Creates a new reference to the assigned peripherals with a shorter lifetime.
            ///
            /// Use this method if you would like to keep working with the peripherals after
            /// you dropped the drivers that consume this.
            pub fn reborrow(&mut self) -> $struct_name<'_> {
                $struct_name {
                    $($group_name: self.$group_name.reborrow()),*
                }
            }
        }

        /// Extracts resources from the `Peripherals` struct.
        #[macro_export]
        macro_rules! split_resources {
            ($peris:ident) => {
                $struct_name {
                    $($group_name: $group_struct {
                        $($resource_name: $peris.$resource_field),*
                    }),*
                }
            }
        }
    };
}

assign_resources! {
    pub Resources<'d> {
        i2c: I2cResources<'d> {
            sda: GPIO45,
            scl: GPIO46,
            i2c: I2C0,
            reset: GPIO9,
        },
        system: SystemResources<'d> {
            int: GPIO10,
        },
        led: LedResources<'d> {
            data: GPIO21,
        },
        top_board: TopBoardResources<'d> {
            hs_1: GPIO8,
            hs_2: GPIO7,
            hs_3: GPIO2,
            hs_4: GPIO1,
        },
        display: DisplayResources<'d> {
            spi: SPI2,
            dma: DMA_CH0,
        },
        hexpansion_a: HexpansionAResources<'d> {
            hs_1: GPIO39,
            hs_2: GPIO40,
            hs_3: GPIO41,
            hs_4: GPIO42,
        },
        hexpansion_b: HexpansionBResources<'d> {
            hs_1: GPIO35,
            hs_2: GPIO36,
            hs_3: GPIO37,
            hs_4: GPIO38,
        },
        hexpansion_c: HexpansionCResources<'d> {
            hs_1: GPIO34,
            hs_2: GPIO33,
            hs_3: GPIO47,
            hs_4: GPIO48,
        },
        hexpansion_d: HexpansionDResources<'d> {
            hs_1: GPIO11,
            hs_2: GPIO14,
            hs_3: GPIO13,
            hs_4: GPIO12,
        },
        hexpansion_e: HexpansionEResources<'d> {
            hs_1: GPIO18,
            hs_2: GPIO16,
            hs_3: GPIO15,
            hs_4: GPIO17,
        },
        hexpansion_f: HexpansionFResources<'d> {
            hs_1: GPIO3,
            hs_2: GPIO4,
            hs_3: GPIO5,
            hs_4: GPIO6,
        },
    }
}
