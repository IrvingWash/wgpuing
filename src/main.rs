fn main() -> Result<(), String> {
    pollster::block_on(wgpuing::run())
}
