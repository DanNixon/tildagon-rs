esp_hal::assign_resources! {
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
