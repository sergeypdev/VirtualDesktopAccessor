//! Windows 10, version 1809
//!
//! From Wikipedia:
//!
//! > Windows 10 October 2018 Update (also known as version 1809 and codenamed
//! > "Redstone 5") is the sixth major update to Windows 10 and the fifth in a
//! > series of updates under the Redstone codenames. It carries the build
//! > number 10.0.17763.
//!
//! [Windows 10 build 17663 (rs_onecore_webplat_comp_dev4) -
//! BetaWiki](https://betawiki.net/wiki/Windows_10_build_17663_(rs_onecore_webplat_comp_dev4)):
//!
//! > Windows 10 build 17663 (rs_onecore_webplat_comp_dev4) is an unleaked build
//! > of Windows 10 October 2018 Update. It was showcased in the session
//! > "Building powerful desktop and MR applications with new windowing APIs" at
//! > the Microsoft Build 2018 conference.
//!
//! # Interface definitions
//!
//! - [Fixed for 1809 ·
//! Ciantic/VirtualDesktopAccessor@4fc1d8e](https://github.com/Ciantic/VirtualDesktopAccessor/commit/4fc1d8e5c74e1a422f5b7fc58cc6674f13718f3e)
//! - Name of new method was later specified by:
//!   [VirtualDesktopAccessor/src/interfaces.rs at
//!   126b9e04f4f01d434af06c20d8200d0659547774 ·
//!   Ciantic/VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor/blob/126b9e04f4f01d434af06c20d8200d0659547774/src/interfaces.rs#L357-L360)

use super::*;
use build_17134 as build_prev;

// These interfaces haven't changed since the previous version:
build_prev::IApplicationView!("871F602A-2B58-42B4-8C4B-6C43D642C06F");
build_prev::IVirtualDesktop!("FF72FFDD-BE7E-43FC-9C03-AD81681E88E4");
build_prev::IVirtualDesktopManagerInternal!("F31574D6-B682-4CDC-BD56-1827860ABEC6");
build_prev::IVirtualDesktopNotification!("C179334C-4295-40D3-BEA1-C654D965605A");
build_prev::IVirtualDesktopNotificationService!("0CD45E71-D927-4F15-8B0A-8FEF525337BF");
build_prev::IVirtualDesktopPinnedApps!("4CE81583-1E4C-4632-A621-07A53543148F");

// But these interfaces have different methods:

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IApplicationViewCollection,
        iid: "1841C6D7-4F9D-42C0-AF41-8747538F10E5", // Different from previous version
    },
    {
        pub unsafe trait IApplicationViewCollection: IUnknown {
            pub unsafe fn get_views(&self, out_views: *mut IObjectArray) -> HRESULT;

            pub unsafe fn get_views_by_zorder(&self, out_views: *mut IObjectArray) -> HRESULT;

            pub unsafe fn get_views_by_app_user_model_id(
                &self,
                id: PCWSTR,
                out_views: *mut IObjectArray,
            ) -> HRESULT;

            pub unsafe fn get_view_for_hwnd(
                &self,
                window: HWND,
                out_view: *mut Option<IApplicationView>,
            ) -> HRESULT;

            pub unsafe fn get_view_for_application(
                &self,
                app: IImmersiveApplication,
                out_view: *mut IApplicationView,
            ) -> HRESULT;

            pub unsafe fn get_view_for_app_user_model_id(
                &self,
                id: PCWSTR,
                out_view: *mut IApplicationView,
            ) -> HRESULT;

            pub unsafe fn get_view_in_focus(&self, out_view: *mut IApplicationView) -> HRESULT;

            // This method is new in Windows 1809:
            pub unsafe fn try_get_last_active_visible_view(
                &self,
                out_view: *mut IApplicationView,
            ) -> HRESULT;

            pub unsafe fn refresh_collection(&self) -> HRESULT;

            pub unsafe fn register_for_application_view_changes(
                &self,
                listener: IApplicationViewChangeListener,
                out_id: *mut DWORD,
            ) -> HRESULT;

            // Note: removed "register_for_application_view_position_changes"
            // that used to be here.

            pub unsafe fn unregister_for_application_view_changes(&self, id: DWORD) -> HRESULT;
        }
    }
);
