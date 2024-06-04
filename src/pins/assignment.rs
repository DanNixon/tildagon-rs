macro_rules! assign_io_pins {
    (
        $struct_name:ident {
            $(
                $field:ident : $sub_struct:ident {
                    $(
                        $sub_field:ident : <$addr:expr, $port:ident, $pin:expr,>,
                    )*
                },
            )*
        }
    ) => {
        pub struct $struct_name {
            $(
                pub $field: $sub_struct,
            )*
        }

        $(
            pub struct $sub_struct {
                $(
                    pub $sub_field: $crate::pins::pin::Pin<$addr, {$crate::pins::aw9523b::Port::$port}, $pin>,
                )*
            }
        )*

        impl Pins {
            pub(super) fn new() -> Self {
                Self {
                    $(
                        $field: $sub_struct {
                            $(
                                $sub_field: $crate::pins::pin::Pin::new(),
                            )*
                        },
                    )*
                }
            }
        }
    };
}

assign_io_pins! {
    Pins {
        other: OtherPins {
            vbus_sw: <0x5A, Port0, 4,>,
            usb_select: <0x5A, Port0, 5,>,
            accel_int: <0x58, Port0, 1,>,
        },
        led: LedPins {
            power_enable: <0x5A, Port0, 2,>,
        },
        top_board: TopBoardPins {
            ls_1: <0x5A, Port1, 7,>,
            ls_2: <0x5A, Port1, 6,>,
        },
        hexpansion_detect: HexpansionDetectPins {
            a: <0x5A, Port1, 4,>,
            b: <0x5A, Port1, 5,>,
            c: <0x59, Port1, 0,>,
            d: <0x59, Port1, 1,>,
            e: <0x59, Port1, 2,>,
            f: <0x59, Port1, 3,>,
        },
        button: ButtonPins {
            btn1: <0x5A, Port0, 6,>,
            btn2: <0x5A, Port0, 7,>,
            btn3: <0x59, Port0, 0,>,
            btn4: <0x59, Port0, 1,>,
            btn5: <0x59, Port0, 2,>,
            btn6: <0x59, Port0, 3,>,
        },
        hexpansion_a: HexpansionAPins {
            ls_1: <0x5A, Port0, 3,>,
            ls_2: <0x5A, Port1, 0,>,
            ls_3: <0x5A, Port1, 1,>,
            ls_4: <0x5A, Port1, 2,>,
            ls_5: <0x5A, Port1, 3,>,
        },
        hexpansion_b: HexpansionBPins {
            ls_1: <0x5A, Port0, 0,>,
            ls_2: <0x5A, Port0, 1,>,
            ls_3: <0x59, Port1, 5,>,
            ls_4: <0x59, Port1, 6,>,
            ls_5: <0x59, Port1, 7,>,
        },
        hexpansion_c: HexpansionCPins {
            ls_1: <0x59, Port0, 4,>,
            ls_2: <0x59, Port0, 5,>,
            ls_3: <0x59, Port0, 6,>,
            ls_4: <0x59, Port0, 7,>,
            ls_5: <0x59, Port1, 4,>,
        },
        hexpansion_d: HexpansionDPins {
            ls_1: <0x58, Port1, 0,>,
            ls_2: <0x58, Port1, 1,>,
            ls_3: <0x58, Port1, 2,>,
            ls_4: <0x58, Port1, 3,>,
            ls_5: <0x58, Port0, 0,>,
        },
        hexpansion_e: HexpansionEPins {
            ls_1: <0x58, Port0, 2,>,
            ls_2: <0x58, Port0, 3,>,
            ls_3: <0x58, Port0, 4,>,
            ls_4: <0x58, Port0, 5,>,
            ls_5: <0x58, Port0, 6,>,
        },
        hexpansion_f: HexpansionFPins {
            ls_1: <0x58, Port0, 7,>,
            ls_2: <0x58, Port1, 4,>,
            ls_3: <0x58, Port1, 5,>,
            ls_4: <0x58, Port1, 6,>,
            ls_5: <0x58, Port1, 7,>,
        },
    }
}
