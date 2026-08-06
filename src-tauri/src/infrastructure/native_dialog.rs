use std::path::PathBuf;

use crate::phase09::error::{Phase09Error, Phase09Result};

pub fn save_file(default_name: &str, default_extension: &str) -> Phase09Result<PathBuf> {
    platform::save_file(default_name, default_extension)
}

pub fn open_file() -> Phase09Result<PathBuf> {
    platform::open_file()
}

#[cfg(not(windows))]
mod platform {
    use super::*;

    pub fn save_file(_default_name: &str, _default_extension: &str) -> Phase09Result<PathBuf> {
        Err(Phase09Error::new(
            "PLATFORM_UNSUPPORTED",
            "Native file dialogs are available only on Windows.",
            false,
        ))
    }

    pub fn open_file() -> Phase09Result<PathBuf> {
        Err(Phase09Error::new(
            "PLATFORM_UNSUPPORTED",
            "Native file dialogs are available only on Windows.",
            false,
        ))
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;

    use super::*;
    use windows::{
        core::{HRESULT, HSTRING, PWSTR},
        Win32::{
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            },
            UI::Shell::{
                FileOpenDialog, FileSaveDialog, IFileOpenDialog, IFileSaveDialog, IShellItem,
                SIGDN_FILESYSPATH,
            },
        },
    };

    const ERROR_CANCELLED_HRESULT: HRESULT = HRESULT(0x8007_04C7u32 as i32);
    const RPC_E_CHANGED_MODE: HRESULT = HRESULT(0x8001_0106u32 as i32);

    pub fn save_file(default_name: &str, default_extension: &str) -> Phase09Result<PathBuf> {
        let _apartment = ComApartment::enter()?;
        unsafe {
            let dialog: IFileSaveDialog =
                CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|_| dialog_failure())?;
            dialog
                .SetFileName(&HSTRING::from(default_name))
                .map_err(|_| dialog_failure())?;
            dialog
                .SetDefaultExtension(&HSTRING::from(default_extension))
                .map_err(|_| dialog_failure())?;
            show_and_resolve(dialog.Show(None), || dialog.GetResult())
        }
    }

    pub fn open_file() -> Phase09Result<PathBuf> {
        let _apartment = ComApartment::enter()?;
        unsafe {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|_| dialog_failure())?;
            show_and_resolve(dialog.Show(None), || dialog.GetResult())
        }
    }

    unsafe fn show_and_resolve(
        shown: windows::core::Result<()>,
        get_result: impl FnOnce() -> windows::core::Result<IShellItem>,
    ) -> Phase09Result<PathBuf> {
        match shown {
            Ok(()) => resolve_item(get_result().map_err(|_| dialog_failure())?),
            Err(error) if error.code() == ERROR_CANCELLED_HRESULT => Err(Phase09Error::new(
                "DIALOG_CANCELLED",
                "The file dialog was cancelled.",
                false,
            )),
            Err(_) => Err(dialog_failure()),
        }
    }

    unsafe fn resolve_item(item: IShellItem) -> Phase09Result<PathBuf> {
        let value: PWSTR = item
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(|_| dialog_failure())?;
        let decoded = value.to_string().map_err(|_| dialog_failure());
        CoTaskMemFree(Some(value.0 as *const c_void));
        decoded.map(PathBuf::from)
    }

    fn dialog_failure() -> Phase09Error {
        Phase09Error::new(
            "DIALOG_FAILED",
            "The Windows file dialog could not be completed.",
            true,
        )
    }

    struct ComApartment {
        owned: bool,
    }

    impl ComApartment {
        fn enter() -> Phase09Result<Self> {
            let result = unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
            };
            match result {
                Ok(()) => Ok(Self { owned: true }),
                Err(error) if error.code() == RPC_E_CHANGED_MODE => Ok(Self { owned: false }),
                Err(_) => Err(dialog_failure()),
            }
        }
    }

    impl Drop for ComApartment {
        fn drop(&mut self) {
            if self.owned {
                unsafe { CoUninitialize() };
            }
        }
    }
}
