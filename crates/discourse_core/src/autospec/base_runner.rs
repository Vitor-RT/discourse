pub trait BaseRunner {
    fn start(&mut self) {}

    fn running(&self) -> bool {
        true
    }

    fn run(&self, _specs: &str) -> i32 {
        0
    }

    fn reload(&mut self) {}

    fn abort(&self) {}

    fn failed_specs(&self) -> Vec<String> {
        vec![]
    }

    fn stop(&self) {}
}
