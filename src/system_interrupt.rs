use crate::resources::SystemResources;
use defmt::info;
use esp_hal::gpio::Input;

pub struct SystemInterrupt {
    int: Input<'static>,
}

impl SystemInterrupt {
    pub fn new(r: SystemResources<'static>) -> Self {
        let int = Input::new(r.int, Default::default());
        Self { int }
    }

    pub async fn wait_for_interrupt(&mut self) {
        self.int.wait_for_falling_edge().await;
        info!("System interrupt trigger");
    }
}
