use mokey_core::Rect;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenError {
    #[error("screen capture failed: {0}")]
    Failed(String),
    #[error("screen capture not supported on this platform")]
    Unsupported,
}

/// Raw BGRA pixel data (not premultiplied), row-major from top-left.
pub struct ScreenImage {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

/// Captures a screen region in physical pixels.
pub fn capture_region(rect: Rect) -> Result<ScreenImage, ScreenError> {
    #[cfg(windows)]
    {
        crate::screen::windows::capture(rect)
    }
    #[cfg(not(windows))]
    {
        let _ = rect;
        Err(ScreenError::Unsupported)
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Graphics::Gdi::{
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
        DeleteDC, DeleteObject, DIB_RGB_COLORS, GetDC, GetDIBits, ReleaseDC, SelectObject, SRCCOPY,
        HGDIOBJ,
    };

    pub fn capture(rect: Rect) -> Result<ScreenImage, ScreenError> {
        let w = rect.w.max(0) as u32;
        let h = rect.h.max(0) as u32;
        if w == 0 || h == 0 {
            return Err(ScreenError::Failed("empty region".into()));
        }

        unsafe {
            let hdc_screen = GetDC(std::ptr::null_mut());
            if hdc_screen == std::ptr::null_mut() {
                return Err(ScreenError::Failed("GetDC failed".into()));
            }
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            let hbmp = CreateCompatibleBitmap(hdc_screen, w as i32, h as i32);
            if hdc_mem == std::ptr::null_mut() || hbmp == std::ptr::null_mut() {
                ReleaseDC(std::ptr::null_mut(), hdc_screen);
                return Err(ScreenError::Failed("CreateCompatible* failed".into()));
            }
            let hbmp_old = SelectObject(hdc_mem, hbmp as HGDIOBJ);
            let ok = BitBlt(
                hdc_mem, 0, 0, w as i32, h as i32, hdc_screen, rect.x, rect.y, SRCCOPY,
            );
            if ok == 0 {
                SelectObject(hdc_mem, hbmp_old);
                DeleteObject(hbmp as _);
                DeleteDC(hdc_mem);
                ReleaseDC(std::ptr::null_mut(), hdc_screen);
                return Err(ScreenError::Failed("BitBlt failed".into()));
            }

            let mut bgra = vec![0u8; (w * h * 4) as usize];
            let mut bmi: BITMAPINFO = zeroed();
            bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = w as i32;
            bmi.bmiHeader.biHeight = -(h as i32);
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;

            let got = GetDIBits(
                hdc_mem,
                hbmp,
                0,
                h,
                bgra.as_mut_ptr() as *mut _,
                &mut bmi,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc_mem, hbmp_old);
            DeleteObject(hbmp as _);
            DeleteDC(hdc_mem);
            ReleaseDC(std::ptr::null_mut(), hdc_screen);

            if got == 0 {
                return Err(ScreenError::Failed("GetDIBits failed".into()));
            }
            Ok(ScreenImage { width: w, height: h, bgra })
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn capture_region_returns_pixels() {
        let img = capture_region(Rect { x: 0, y: 0, w: 50, h: 40 }).expect("capture should work");
        assert_eq!(img.width, 50);
        assert_eq!(img.height, 40);
        assert_eq!(img.bgra.len(), 50 * 40 * 4);
    }
}
