use crate::error::BackendError;

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Monitor bounds in physical pixels.
    pub rect: mokey_core::Rect,
    /// DPI scale factor (physical pixels per logical point).
    pub scale: f64,
}

/// Enumerate monitors. Coordinates are in physical pixels.
#[cfg(windows)]
pub fn list() -> Result<Vec<MonitorInfo>, BackendError> {
    use windows_sys::Win32::Foundation::{BOOL, LPARAM, RECT};
    use windows_sys::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
    };
    use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

    let mut monitors: Vec<MonitorInfo> = Vec::new();

    unsafe extern "system" fn callback(
        hmon: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(data as *mut Vec<MonitorInfo>);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmon, &mut info) != 0 {
            let rc = info.rcMonitor;
            let mut dpi_x: u32 = 0;
            let mut dpi_y: u32 = 0;
            let scale = if GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) == 0 {
                dpi_x as f64 / 96.0
            } else {
                1.0
            };
            monitors.push(MonitorInfo {
                rect: mokey_core::Rect {
                    x: rc.left,
                    y: rc.top,
                    w: (rc.right - rc.left) as u32,
                    h: (rc.bottom - rc.top) as u32,
                },
                scale,
            });
        }
        1
    }

    let ok = unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(callback),
            &mut monitors as *mut _ as isize,
        )
    };
    if ok == 0 {
        return Err(BackendError::Input("EnumDisplayMonitors failed".into()));
    }
    Ok(monitors)
}

/// Enumerate monitors (Linux backends arrive in Phase 2/3).
#[cfg(unix)]
pub fn list() -> Result<Vec<MonitorInfo>, BackendError> {
    Err(BackendError::UnsupportedPlatform(
        "mokey has not been ported to this Linux desktop yet".into(),
    ))
}
