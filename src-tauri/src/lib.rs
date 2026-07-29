fn configure_application<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
}

fn application_builder() -> tauri::Builder<tauri::Wry> {
    configure_application(tauri::Builder::default())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    application_builder()
        .run(tauri::generate_context!())
        .expect("failed to run the POSMAN desktop shell");
}

#[cfg(test)]
mod tests {
    use super::configure_application;

    #[test]
    fn application_setup_builds_with_mock_runtime() {
        let _application = configure_application(tauri::test::mock_builder())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("failed to build the POSMAN shell with Tauri's mock runtime");
    }
}
