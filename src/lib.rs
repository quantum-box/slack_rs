pub mod socket_mode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_mode_module_exists() {
        // モジュールが正しくエクスポートされていることを確認
        let _ = socket_mode::run_socket_mode;
    }
}
