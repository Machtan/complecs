pub extern crate froggy;

#[macro_use]
pub mod macros;
pub mod traits;
pub use traits::AddEntityToStore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
