mod device;
mod controller;

#[cfg(test)]
mod tests {
    use crate::controller::Controller;
    use crate::device::memory::Memory;

    #[test]
    fn can_add_device_to_controller() {
        let mut controller = Controller::new();
        let device = Memory::new();
        controller.add_device("mem1".to_owned(), Box::new(device));
    }
}
