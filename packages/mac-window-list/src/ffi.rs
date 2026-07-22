use core_foundation::array::CFArrayRef;

pub(crate) type CGWindowID = u32;
pub(crate) type CGWindowListOption = u32;

pub(crate) const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: CGWindowListOption = 1 << 0;
pub(crate) const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: CGWindowListOption = 1 << 4;
pub(crate) const K_CG_NULL_WINDOW_ID: CGWindowID = 0;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub(crate) fn CGWindowListCopyWindowInfo(
        option: CGWindowListOption,
        relative_to_window: CGWindowID,
    ) -> CFArrayRef;
}
