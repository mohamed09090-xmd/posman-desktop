fn application_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    application_builder()
        .run(tauri::generate_context!())
        .expect("failed to run the POSMAN desktop shell");
}

#[cfg(test)]
mod tests {
    use super::application_builder;

    #[test]
    fn application_builder_is_constructible() {
        let _builder = application_builder();
    }
}
