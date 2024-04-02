//! Interface abstraction that allow interacting with COM interfaces even when
//! we don't know their IID and what exact methods they have until runtime.
//!
//! The interface types in this module is similar to the ones in the version
//! specific modules except they don't implement `ComInterface` since their IID
//! isn't known at compile time.

use super::*;

use crate::comobjects::HRESULTHelpers;
use core::{ffi::c_void, marker::PhantomData};
use windows::{
    core::{ComInterface, Interface, GUID, HRESULT, HSTRING},
    Win32::{
        Foundation::{E_NOTIMPL, HWND},
        UI::Shell::Common::IObjectArray,
    },
};

macro_rules! declare_WindowsVersion {
    (versions = {$($version:ident,)*},) => {
        /// Indicates different Windows versions that have different Virtual Desktop
        /// interfaces.
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        #[allow(non_camel_case_types)]
        enum WindowsVersion {
            $($version,)*
        }
        impl WindowsVersion {
            const ALL: &'static [Self] = &[$(Self::$version,)*];
            const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$version => stringify!($version),)*
                }
            }
        }
        impl Default for WindowsVersion {
            fn default() -> Self {
                *Self::ALL.last().expect("No Windows version is supported")
            }
        }
    };
}
with_versions!(declare_WindowsVersion);

// Check that the versions are sorted when they were declared, this is a bit
// slow currently so it has been disabled:
// impl WindowsVersion {
//      #[allow(dead_code)]
//      const fn has_sorted_versions() -> bool {
//          let mut prev = 0;
//          let ix = 0;
//          let arr = Self::ALL;
//          while ix < arr.len() {
//              let ver = arr[ix].windows_build();
//              if ver < prev {
//                  return false;
//              }
//              prev = ver;
//          }
//          true
//      }
//      /// Code from <https://docs.rs/static_assertions/latest/src/static_assertions/const_assert.rs.html#52-57>
//      const _ASSERT_SORTED_VERSIONS: [(); 0 - !Self::has_sorted_versions() as usize] = [];
//      const fn windows_build(&self) -> u32 {
//          let name = self.as_str();
//          let bytes = name.as_bytes();
//          let mut ix = 0;
//          while ix < bytes.len() && !u8::is_ascii_digit(&bytes[ix]) {
//              ix += 1;
//          }
//          let mut parsed = 0;
//          while ix < bytes.len() {
//              parsed *= 10;
//              parsed += (bytes[ix] - b'0') as u32;
//              ix += 1;
//          }
//          parsed
//      }
// }
impl WindowsVersion {
    fn windows_build(&self) -> u32 {
        let (_, ver) = self
            .as_str()
            .split_once('_')
            .expect("Module name didn't contain an underscore");
        ver.parse()
            .expect("Failed to parse module suffix as build version")
    }
    /// Get info about the current Windows version. Only differentiates between
    /// Windows versions that have different virtual desktop interfaces.
    ///
    /// # Determining Windows Version
    ///
    /// We could use the [`GetVersionExW` function
    /// (sysinfoapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getversionexw),
    /// but it is deprecated after Windows 8.1. It also changes behavior depending
    /// on what manifest is embedded in the executable.
    ///
    /// That pages links to [Version Helper functions - Win32
    /// apps](https://learn.microsoft.com/en-us/windows/win32/sysinfo/version-helper-apis)
    /// where we are linked to the [`IsWindowsVersionOrGreater` function
    /// (versionhelpers.h)](https://learn.microsoft.com/en-us/windows/win32/api/VersionHelpers/nf-versionhelpers-iswindowsversionorgreater)
    /// and the [`VerifyVersionInfoA` function
    /// (winbase.h)](https://learn.microsoft.com/en-us/windows/win32/api/Winbase/nf-winbase-verifyversioninfoa)
    /// that it uses internally (though the later function is deprecated in Windows
    /// 10).
    ///
    /// We can use `RtlGetVersion` [RtlGetVersion function (wdm.h) - Windows
    /// drivers](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-rtlgetversion?redirectedfrom=MSDN)
    /// as mentioned at [c++ - Detecting Windows 10 version - Stack
    /// Overflow](https://stackoverflow.com/questions/36543301/detecting-windows-10-version/36545162#36545162).
    ///
    /// # `windows` API References
    ///
    /// - [GetVersionExW in windows::Win32::System::SystemInformation -
    ///   Rust](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/SystemInformation/fn.GetVersionExW.html)
    ///   - Affected by manifest.
    /// - [RtlGetVersion in windows::Wdk::System::SystemServices -
    ///   Rust](https://microsoft.github.io/windows-docs-rs/doc/windows/Wdk/System/SystemServices/fn.RtlGetVersion.html)
    ///   - Always returns the correct version.
    pub fn get() -> Self {
        static INIT: std::sync::OnceLock<WindowsVersion> = std::sync::OnceLock::new();
        *INIT.get_or_init(|| {
            let mut version: windows::Win32::System::SystemInformation::OSVERSIONINFOW =
                Default::default();
            version.dwOSVersionInfoSize = core::mem::size_of_val(&version) as u32;
            let res = unsafe { windows::Wdk::System::SystemServices::RtlGetVersion(&mut version) };
            if res.is_err() {
                return Default::default();
            }
            Self::ALL
                .iter()
                .copied()
                .map(|v| (v, v.windows_build()))
                // Only consider COM interfaces from previous or current build:
                .filter(|(_, build_ver)| *build_ver <= version.dwBuildNumber)
                // Then find the latest one:
                .max_by_key(|(_, build_ver)| *build_ver)
                .map(|(v, _)| v)
                .unwrap_or_default()
        })
    }
}

/// Do an action with the type of the actual COM Interface on this Windows
/// version.
///
/// This is implemented for the more generic COM interfaces that don't know
/// their IID at compile time. Those implementations will put a bound on `F` so
/// that it must accept all concrete COM types that might be used.
pub trait WithVersionedType<F, R> {
    /// Invokes the callback with the COM interface type of this Windows
    /// version. Return `None` if the interface doesn't exist on this platform.
    fn with_versioned_type(callback: F) -> Option<R>;
}
/// A callback that will be invoked with the actual COM interface type for a
/// specific Windows version.
///
/// Implement this when you want to make use of a concrete COM interface type.
pub trait WithVersionedTypeCallback<T: ComInterface, R> {
    fn call(self) -> R;
}

pub(crate) trait ForwardArg<T> {
    fn forward(self) -> T;
}
impl<T> ForwardArg<T> for T {
    fn forward(self) -> Self {
        self
    }
}

/// Generates support code for a COM interface.
///
/// Syntax is: `as CreatedEnumName for InterfaceName in $(all)? [version_module_1, version_module_2 $(,)?]`
macro_rules! support_interface {
    (@inner {$dollar:tt} as $state:ident for $name:ident in $(all $(@ $all:tt)?)? [$($version:ident),* $(,)?]) => {
        $(
            // assert_eq_size from static_assertions crate
            const _: fn() = || {
                // We need this since we transmute and pointer cast between the two types.
                let _ = core::mem::transmute::<$name, self::$version::$name>;
            };
        )*

        // Maybe enforce that all build versions are supported by this interface:
        #[allow(unreachable_patterns)]
        const _: fn(WindowsVersion) = |version: WindowsVersion| {
            match version {
                $(WindowsVersion::$version => (),)*
                // If there is no "all" word in the macro input then:
                //   all() => true
                // Otherwise:
                //   all(false) => false
                //   any() => false
                //   all(any()) => false
                // And the default macro arm will be hidden.
                #[cfg(all($(any() $($all)?)?))]
                _ => (),
            }
        };

        /// An enum with one variant per Windows version that is supported by
        /// this interface. Use the `from_typed` function to construct this
        /// type.
        #[allow(non_camel_case_types)]
        enum $state<'a> {
            $( $version(ComIn<'a, self::$version::$name>) ),*
        }
        impl<'a> $state<'a> {
            fn from_typed(data: &'a $name) -> Self {
                unsafe { Self::from_raw(&data.0) }
            }
            /// # Safety
            ///
            /// The COM object must implement the expected interface.
            #[allow(unreachable_patterns)]
            unsafe fn from_raw(data: &'a IUnknown) -> Self {
                let win_ver = WindowsVersion::get();
                match win_ver {
                    $(WindowsVersion::$version => $state::$version(core::mem::transmute_copy::<IUnknown, ComIn<'_, _>>(data)),)*
                    _ => unreachable!("Tried to cast into a COM interface that wasn't available for the current Windows version"),
                }
            }
        }
        impl $name {
            /// Convert from a raw pointer to the COM interface.
            ///
            /// # Safety
            ///
            /// The pointer must be an instance of the COM interface indicated
            /// by the `IID` method.
            pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
                Self(IUnknown::from_raw(ptr))
            }
            pub fn as_raw(&self) -> *mut c_void {
                self.0.as_raw()
            }

            /// The IID for the COM interface that is supported by this
            /// platform, return a zeroed GUID if the interface isn't supported.
            #[allow(non_snake_case, unreachable_patterns)]
            pub unsafe fn IID() -> GUID {
                match WindowsVersion::get() {
                    $(WindowsVersion::$version => self::$version::$name::IID,)*
                    _ => GUID::zeroed(),
                }
            }
        }
        /// Allow putting the abstract type in the `ComIn` wrapper type.
        unsafe impl PointerRepr for $name {
            fn as_pointer_repr(&self) -> *mut c_void {
                windows::core::Interface::as_raw(&self.0)
            }
        }
        /// Allow direct access to the wrapped COM interface type if required.
        impl<F, R> WithVersionedType<F, R> for $name
        where
            $(
                F: WithVersionedTypeCallback<self::$version::$name, R>,
            )*
        {
            #[allow(unreachable_patterns)]
            fn with_versioned_type(callback: F) -> Option<R> {
                match WindowsVersion::get() {
                    $(WindowsVersion::$version => Some(<F as WithVersionedTypeCallback<self::$version::$name, R>>::call(callback)),)*
                    _ => None,
                }
            }
        }
        // From implementations to make forwarding arguments to versioned
        // interface methods easier:
        $(
            /// Versioned -> Abstract
            impl From<self::$version::$name> for $name {
                fn from(v: self::$version::$name) -> Self {
                    debug_assert_eq!(
                        WindowsVersion::get(),
                        WindowsVersion::$version,
                        "if we have an COM interface for a specific Windows version then we must already have ensured that it is actually the Windows version the user has"
                    );
                    Self(v.into())
                }
            }
            /// &Versioned -> &Abstract
            impl<'a> From<&'a self::$version::$name> for &'a $name {
                fn from(v: &'a self::$version::$name) -> Self {
                    debug_assert_eq!(
                        WindowsVersion::get(),
                        WindowsVersion::$version,
                        "if we have an COM interface for a specific Windows version then we must already have ensured that it is actually the Windows version the user has"
                    );
                    // Safety: both types are just transparent wrappers over a
                    // raw pointer and we don't drop either of them.
                    unsafe {
                        &*(v as *const self::$version::$name as *const $name)
                    }
                }
            }
            /// ComIn<Versioned> -> ComIn<Abstract>
            impl<'a> From<ComIn<'a, self::$version::$name>> for ComIn<'a, $name> {
                fn from(v: ComIn<'a, self::$version::$name>) -> Self {
                    ComIn::new(<&$name as From<_>>::from(ComIn::into_ref(&v)))
                }
            }
            /// Abstract -> Versioned (fallible)
            impl From<$name> for self::$version::$name {
                fn from(v: $name) -> Self {
                    assert_eq!(WindowsVersion::get(), WindowsVersion::$version);
                    // Safety: interpret the wrapped raw pointer as the specific COM interface.
                    unsafe { core::mem::transmute(v.0) }
                }
            }
            /// ComIn<Abstract> -> ComIn<Versioned> (fallible)
            impl<'a> From<ComIn<'a, $name>> for ComIn<'a, self::$version::$name> {
                #[allow(irrefutable_let_patterns)]
                fn from(v: ComIn<'a, $name>) -> Self {
                    if let $state::$version(v) = $state::from_typed(ComIn::into_ref(&v)) {
                        v
                    } else {
                        unreachable!("requested a COM interface for a different Windows version than the one that was installed");
                    }
                }
            }
            /// ComIn<Abstract> -> ComIn<Versioned> (fallible)
            impl<'a> ForwardArg<ComIn<'a, self::$version::$name>> for ComIn<'a, $name> {
                fn forward(self) -> ComIn<'a, self::$version::$name> {
                    self.into()
                }
            }
            /// *mut Option<Abstract> -> *mut Option<Versioned>
            impl ForwardArg<*mut Option<self::$version::$name>> for *mut Option<$name> {
                fn forward(self) -> *mut Option<self::$version::$name> {
                    self as *mut _
                }
            }
            /// *mut Abstract -> *mut Versioned
            impl ForwardArg<*mut self::$version::$name> for *mut $name {
                fn forward(self) -> *mut self::$version::$name {
                    self as *mut _
                }
            }
        )*
        /// Preform the same action for each version of the wrapped COM interface.
        ///
        /// Syntax: GeneralType, |versioned: versioned_mod::VersionedType| block_of_code
        ///
        /// Note: named the same as the interface to allow for easier usage with macros.
        #[allow(unused_macros)]
        macro_rules! $name {
            (
                $dollar this:expr,
                |$dollar arg:ident
                    $dollar (
                        :
                        $dollar module_name:ident
                        ::
                        $dollar arg_ty:ident
                    )?
                |
                $dollar ($dollar body:tt)*
            ) => {
                match $state::from_typed(&$dollar this) {
                    $(
                        $state::$version($dollar arg) => {
                            $dollar (
                                #[allow(unused_imports)]
                                use self::$version as $dollar module_name;
                                #[allow(unused_imports)]
                                use self::$version::$name as $dollar arg_ty;
                            )?
                            $dollar ($dollar body)*
                        },
                    )*
                }
            }
        }
    };
    // Pass an escaped dollar sign to the real macro so that we can construct a
    // new macro later:
    (as $state:ident for $name:ident in $($in:tt)*) => {
        support_interface! { @inner {$} as $state for $name in $($in)* }
    };
}

/// Implement a method by calling the same method on the Windows version
/// dependant COM interface.
macro_rules! forward_call {
    (
        #[forward_for = $name:ident]
        $( #[$attr:meta] )*
        $pub:vis
        $(unsafe $(@ $unsafe:tt)?)?
        fn $fname:ident (
            &$self_:ident $(,)? $( $arg_name:ident : $ArgTy:ty ),* $(,)?
        ) -> $RetTy:ty;
    ) => (
        $( #[$attr] )*
        #[allow(unused_parens)]
        $pub
        $(unsafe $($unsafe)?)?
        fn $fname (
            &$self_, $( $arg_name : $ArgTy ),*
        ) -> $RetTy
        {
            unsafe {
                $name!(
                    $self_,
                    |v| (*v).$fname( $(
                        ForwardArg::forward($arg_name)
                    ),*)
                )
            }
        }
    );
    // No function body => forward the call automatically:
    (@item
        #[forward_for = $name:ident]
        $( #[$attr:meta] )*
        $pub:vis
        $(unsafe $(@ $unsafe:tt)?)?
        fn $fname:ident (
            &$self_:ident $(,)? $( $arg_name:ident : $ArgTy:ty ),* $(,)?
        ) -> $RetTy:ty;
    ) => {
        $( #[$attr] )*
        #[allow(unused_parens)]
        $pub
        $(unsafe $($unsafe)?)?
        fn $fname (
            &$self_, $( $arg_name : $ArgTy ),*
        ) -> $RetTy
        {
            unsafe {
                $name!(
                    $self_,
                    |v| (*v).$fname( $(
                        ForwardArg::forward($arg_name)
                    ),*)
                )
            }
        }
    };
    // Manual body implementation:
    (@item
        #[forward_for = $name:ident]
        $( #[$attr:meta] )*
        $pub:vis
        $(unsafe $(@ $unsafe:tt)?)?
        fn $fname:ident (
            &$self_:ident $(,)? $( $arg_name:ident : $ArgTy:ty ),* $(,)?
        ) -> $RetTy:ty {
            $($body:tt)*
        }
    ) => {
        $( #[$attr] )*
        #[allow(unused_parens)]
        $pub
        $(unsafe $($unsafe)?)?
        fn $fname (
            &$self_, $( $arg_name : $ArgTy ),*
        ) -> $RetTy
        { $($body)* }
    };
    (
        $( #[$attr_impl:meta] )*
        impl $name:ident {
            $(
                $( #[$($attr_item:tt)*] )*
                $pub:vis
                $(unsafe $(@ $unsafe:tt)?)?
                fn $fname:ident $params:tt -> $RetTy:ty $({
                    $($body:tt)*
                })?
                $(; $(<= $semi:tt)?)?
            )*
        }
    ) => {
        $(#[$attr_impl])*
        impl $name {
            $(
                forward_call! {@item
                    #[forward_for = $name]
                    $(#[$($attr_item)*])*
                    $pub
                    $(unsafe $($unsafe)?)?
                    fn $fname $params -> $RetTy
                    $({ $($body)* })?
                    $(; $($semi)?)?
                }
            )*
        }
    };
}

support_interface!(as IApplicationViewInner for IApplicationView in all [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IApplicationView(IUnknown);
#[apply(forward_call)]
impl IApplicationView {
    /* IInspecateble */
    pub unsafe fn get_iids(
        &self,
        out_iid_count: *mut ULONG,
        out_opt_iid_array_ptr: *mut *mut GUID,
    ) -> HRESULT;
    pub unsafe fn get_runtime_class_name(
        &self,
        out_opt_class_name: *mut HSTRING,
    ) -> HRESULT;
    pub unsafe fn get_trust_level(&self, ptr_trust_level: LPVOID) -> HRESULT;

    /* IApplicationView methods */
    pub unsafe fn set_focus(&self) -> HRESULT;
    pub unsafe fn switch_to(&self) -> HRESULT;

    pub unsafe fn try_invoke_back(&self, ptr_async_callback: IAsyncCallback) -> HRESULT;
    pub unsafe fn get_thumbnail_window(&self, out_hwnd: *mut HWND) -> HRESULT;
    pub unsafe fn get_monitor(&self, out_monitors: *mut *mut IImmersiveMonitor) -> HRESULT;
    pub unsafe fn get_visibility(&self, out_int: LPVOID) -> HRESULT;
    pub unsafe fn set_cloak(
        &self,
        application_view_cloak_type: APPLICATION_VIEW_CLOAK_TYPE,
        unknown: INT,
    ) -> HRESULT;
    pub unsafe fn get_position(
        &self,
        unknowniid: *const GUID,
        unknown_array_ptr: LPVOID,
    ) -> HRESULT;
    pub unsafe fn set_position(
        &self,
        view_position: *mut IApplicationViewPosition,
    ) -> HRESULT;
    pub unsafe fn insert_after_window(&self, window: HWND) -> HRESULT;
    pub unsafe fn get_extended_frame_position(&self, rect: *mut RECT) -> HRESULT;
    pub unsafe fn get_app_user_model_id(&self, id: *mut PWSTR) -> HRESULT; // Proc17
    pub unsafe fn set_app_user_model_id(&self, id: PCWSTR) -> HRESULT;
    pub unsafe fn is_equal_by_app_user_model_id(
        &self,
        id: PCWSTR,
        out_result: *mut INT,
    ) -> HRESULT;

    /*** IApplicationView methods ***/
    pub unsafe fn get_view_state(&self, out_state: *mut UINT) -> HRESULT; // Proc20
    pub unsafe fn set_view_state(&self, state: UINT) -> HRESULT; // Proc21
    pub unsafe fn get_neediness(&self, out_neediness: *mut INT) -> HRESULT; // Proc22
    pub unsafe fn get_last_activation_timestamp(
        &self,
        out_timestamp: *mut ULONGLONG,
    ) -> HRESULT;
    pub unsafe fn set_last_activation_timestamp(&self, timestamp: ULONGLONG) -> HRESULT;
    pub unsafe fn get_virtual_desktop_id(&self, out_desktop_guid: *mut GUID) -> HRESULT;
    pub unsafe fn set_virtual_desktop_id(&self, desktop_guid: *const GUID) -> HRESULT;
    pub unsafe fn get_show_in_switchers(&self, out_show: *mut INT) -> HRESULT;
    pub unsafe fn set_show_in_switchers(&self, show: INT) -> HRESULT;
    pub unsafe fn get_scale_factor(&self, out_scale_factor: *mut INT) -> HRESULT;
    pub unsafe fn can_receive_input(&self, out_can: *mut BOOL) -> HRESULT;
    pub unsafe fn get_compatibility_policy_type(
        &self,
        out_policy_type: *mut APPLICATION_VIEW_COMPATIBILITY_POLICY,
    ) -> HRESULT;
    pub unsafe fn set_compatibility_policy_type(
        &self,
        policy_type: APPLICATION_VIEW_COMPATIBILITY_POLICY,
    ) -> HRESULT;
    pub unsafe fn get_position_priority(
        &self,
        out_priority: *mut IShellPositionerPriority,
    ) -> HRESULT;
    pub unsafe fn set_position_priority(
        &self,
        priority: IShellPositionerPriority,
    ) -> HRESULT;

    pub unsafe fn get_size_constraints(
        &self,
        monitor: *mut IImmersiveMonitor,
        out_size1: *mut SIZE,
        out_size2: *mut SIZE,
    ) -> HRESULT;
    pub unsafe fn get_size_constraints_for_dpi(
        &self,
        dpi: UINT,
        out_size1: *mut SIZE,
        out_size2: *mut SIZE,
    ) -> HRESULT;
    pub unsafe fn set_size_constraints_for_dpi(
        &self,
        dpi: *const UINT,
        size1: *const SIZE,
        size2: *const SIZE,
    ) -> HRESULT;

    pub unsafe fn query_size_constraints_from_app(&self) -> HRESULT;
    pub unsafe fn on_min_size_preferences_updated(&self, window: HWND) -> HRESULT;
    pub unsafe fn apply_operation(
        &self,
        operation: *mut IApplicationViewOperation,
    ) -> HRESULT;
    pub unsafe fn is_tray(&self, out_is: *mut BOOL) -> HRESULT;
    pub unsafe fn is_in_high_zorder_band(&self, out_is: *mut BOOL) -> HRESULT;
    pub unsafe fn is_splash_screen_presented(&self, out_is: *mut BOOL) -> HRESULT;
    pub unsafe fn flash(&self) -> HRESULT;
    pub unsafe fn get_root_switchable_owner(
        &self,
        app_view: *mut IApplicationView,
    ) -> HRESULT; // proc45
    pub unsafe fn enumerate_ownership_tree(&self, objects: *mut IObjectArray) -> HRESULT; // proc46

    pub unsafe fn get_enterprise_id(&self, out_id: *mut PWSTR) -> HRESULT; // proc47
    pub unsafe fn is_mirrored(&self, out_is: *mut BOOL) -> HRESULT; //

    pub unsafe fn unknown1(&self, arg: *mut INT) -> HRESULT;
    pub unsafe fn unknown2(&self, arg: *mut INT) -> HRESULT;
    pub unsafe fn unknown3(&self, arg: *mut INT) -> HRESULT;
    pub unsafe fn unknown4(&self, arg: INT) -> HRESULT;
    pub unsafe fn unknown5(&self, arg: *mut INT) -> HRESULT;
    pub unsafe fn unknown6(&self, arg: INT) -> HRESULT;
    pub unsafe fn unknown7(&self) -> HRESULT;
    pub unsafe fn unknown8(&self, arg: *mut INT) -> HRESULT;
    pub unsafe fn unknown9(&self, arg: INT) -> HRESULT;
    pub unsafe fn unknown10(&self, arg: INT, arg2: INT) -> HRESULT;
    pub unsafe fn unknown11(&self, arg: INT) -> HRESULT;
    pub unsafe fn unknown12(&self, arg: *mut SIZE) -> HRESULT;
}

support_interface!(as IVirtualDesktopInner for IVirtualDesktop in [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IVirtualDesktop(IUnknown);
#[apply(forward_call)]
impl IVirtualDesktop {
    pub unsafe fn is_view_visible(
        &self,
        p_view: ComIn<IApplicationView>,
        out_bool: *mut u32,
    ) -> HRESULT;
    pub unsafe fn get_id(&self, out_guid: *mut GUID) -> HRESULT;

    pub unsafe fn get_name(&self, out_string: *mut HSTRING) -> HRESULT {
        match IVirtualDesktopInner::from_typed(self) {
            IVirtualDesktopInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopInner::build_22000(this) => unsafe { this.get_name(out_string) },
        }
    }
    pub unsafe fn get_wallpaper(&self, out_string: *mut HSTRING) -> HRESULT {
        match IVirtualDesktopInner::from_typed(self) {
            IVirtualDesktopInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopInner::build_22000(this) => unsafe { this.get_wallpaper(out_string) },
        }
    }
}

support_interface!(as IApplicationViewCollectionInner for IApplicationViewCollection in all [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IApplicationViewCollection(IUnknown);
#[apply(forward_call)]
impl IApplicationViewCollection {
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
    ) -> HRESULT {
        IApplicationViewCollection!(self, |inner| inner
            .get_view_for_hwnd(window, out_view as *mut _))
    }

    pub unsafe fn get_view_for_application(
        &self,
        app: IImmersiveApplication,
        out_view: *mut IApplicationView,
    ) -> HRESULT {
        IApplicationViewCollection!(self, |inner| inner
            .get_view_for_application(app, out_view as *mut _))
    }

    pub unsafe fn get_view_for_app_user_model_id(
        &self,
        id: PCWSTR,
        out_view: *mut IApplicationView,
    ) -> HRESULT {
        IApplicationViewCollection!(self, |inner| inner
            .get_view_for_app_user_model_id(id, out_view as *mut _))
    }

    pub unsafe fn get_view_in_focus(&self, out_view: &mut Option<IApplicationView>) -> HRESULT {
        unsafe {
            IApplicationViewCollection!(self, |inner| inner
                .get_view_in_focus(out_view as *mut Option<_> as *mut _))
        }
    }

    pub unsafe fn refresh_collection(&self) -> HRESULT;

    pub unsafe fn register_for_application_view_changes(
        &self,
        listener: IApplicationViewChangeListener,
        out_id: &mut DWORD,
    ) -> HRESULT {
        unsafe {
            IApplicationViewCollection!(self, |inner| inner
                .register_for_application_view_changes(listener, out_id))
        }
    }

    pub unsafe fn unregister_for_application_view_changes(&self, id: DWORD) -> HRESULT;
}
impl IApplicationViewCollection {
    pub unsafe fn query_service(provider: &IServiceProvider) -> crate::Result<Self> {
        let mut obj = std::ptr::null_mut::<c_void>();
        unsafe {
            provider
                .query_service(
                    &IApplicationViewCollection::IID(),
                    &IApplicationViewCollection::IID(),
                    &mut obj,
                )
                .as_result()?;
        }
        assert_eq!(obj.is_null(), false);
        unsafe { Ok(IApplicationViewCollection::from_raw(obj)) }
    }
}

support_interface!(as IVirtualDesktopNotificationInner for IVirtualDesktopNotification in all [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IVirtualDesktopNotification(IUnknown);
impl<T> From<T> for IVirtualDesktopNotification
where
    T: IVirtualDesktopNotification_Impl,
{
    fn from(value: T) -> Self {
        match WindowsVersion::get() {
            WindowsVersion::build_10240 => build_10240::IVirtualDesktopNotification::from(
                build_10240::VirtualDesktopNotificationAdaptor { inner: value },
            )
            .into(),
            WindowsVersion::build_22000 => build_22000::IVirtualDesktopNotification::from(
                build_22000::VirtualDesktopNotificationAdaptor { inner: value },
            )
            .into(),
        }
    }
}
#[allow(non_camel_case_types)]
pub trait IVirtualDesktopNotification_Impl {
    unsafe fn virtual_desktop_created(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT;

    unsafe fn virtual_desktop_destroy_begin(
        &self,
        desktop_destroyed: ComIn<IVirtualDesktop>,
        desktop_fallback: ComIn<IVirtualDesktop>,
    ) -> HRESULT;

    unsafe fn virtual_desktop_destroy_failed(
        &self,
        desktop_destroyed: ComIn<IVirtualDesktop>,
        desktop_fallback: ComIn<IVirtualDesktop>,
    ) -> HRESULT;

    unsafe fn virtual_desktop_destroyed(
        &self,
        desktop_destroyed: ComIn<IVirtualDesktop>,
        desktop_fallback: ComIn<IVirtualDesktop>,
    ) -> HRESULT;

    unsafe fn virtual_desktop_moved(
        &self,
        desktop: ComIn<IVirtualDesktop>,
        old_index: i64,
        new_index: i64,
    ) -> HRESULT;

    unsafe fn virtual_desktop_name_changed(
        &self,
        desktop: ComIn<IVirtualDesktop>,
        name: HSTRING,
    ) -> HRESULT;

    unsafe fn view_virtual_desktop_changed(&self, view: ComIn<IApplicationView>) -> HRESULT;

    unsafe fn current_virtual_desktop_changed(
        &self,
        desktop_old: ComIn<IVirtualDesktop>,
        desktop_new: ComIn<IVirtualDesktop>,
    ) -> HRESULT;

    unsafe fn virtual_desktop_wallpaper_changed(
        &self,
        desktop: ComIn<IVirtualDesktop>,
        name: HSTRING,
    ) -> HRESULT;

    unsafe fn virtual_desktop_switched(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT;

    unsafe fn remote_virtual_desktop_connected(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT;
}

support_interface!(as IVirtualDesktopNotificationServiceInner for IVirtualDesktopNotificationService in all [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IVirtualDesktopNotificationService(IUnknown);

#[apply(forward_call)]
impl IVirtualDesktopNotificationService {
    pub unsafe fn register(
        &self,
        notification: *mut std::ffi::c_void,
        out_cookie: *mut DWORD,
    ) -> HRESULT;
    pub unsafe fn unregister(&self, cookie: u32) -> HRESULT;
}
impl IVirtualDesktopNotificationService {
    pub unsafe fn query_service(provider: &IServiceProvider) -> crate::Result<Self> {
        let mut obj = std::ptr::null_mut::<c_void>();
        unsafe {
            provider
                .query_service(
                    &CLSID_IVirtualNotificationService,
                    &IVirtualDesktopNotificationService::IID(),
                    &mut obj,
                )
                .as_result()?;
        }
        assert_eq!(obj.is_null(), false);
        unsafe { Ok(IVirtualDesktopNotificationService::from_raw(obj)) }
    }
}

support_interface!(as IVirtualDesktopManagerInternalInner for IVirtualDesktopManagerInternal in all [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IVirtualDesktopManagerInternal(IUnknown);
#[apply(forward_call)]
impl IVirtualDesktopManagerInternal {
    pub unsafe fn get_desktop_count(&self, out_count: *mut UINT) -> HRESULT;

    pub unsafe fn move_view_to_desktop(
        &self,
        view: ComIn<IApplicationView>,
        desktop: ComIn<IVirtualDesktop>,
    ) -> HRESULT;

    pub unsafe fn can_move_view_between_desktops(
        &self,
        view: ComIn<IApplicationView>,
        can_move: *mut i32,
    ) -> HRESULT {
        unsafe {
            IVirtualDesktopManagerInternal!(self, |i| i
                .can_move_view_between_desktops(view.into(), can_move))
        }
    }

    pub unsafe fn get_current_desktop(&self, out_desktop: *mut Option<IVirtualDesktop>) -> HRESULT;

    pub unsafe fn get_desktops(&self, out_desktops: *mut Option<IObjectArray>) -> HRESULT;

    /// Get next or previous desktop
    ///
    /// Direction values:
    /// 3 = Left direction
    /// 4 = Right direction
    pub unsafe fn get_adjacent_desktop(
        &self,
        in_desktop: ComIn<IVirtualDesktop>,
        direction: UINT,
        out_pp_desktop: *mut Option<IVirtualDesktop>,
    ) -> HRESULT;

    pub unsafe fn switch_desktop(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT;

    pub unsafe fn create_desktop(&self, out_desktop: *mut Option<IVirtualDesktop>) -> HRESULT;

    pub unsafe fn move_desktop(&self, in_desktop: ComIn<IVirtualDesktop>, index: UINT) -> HRESULT {
        match IVirtualDesktopManagerInternalInner::from_typed(self) {
            IVirtualDesktopManagerInternalInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopManagerInternalInner::build_22000(v) => {
                v.move_desktop(in_desktop.into(), index)
            }
        }
    }

    pub unsafe fn remove_desktop(
        &self,
        destroy_desktop: ComIn<IVirtualDesktop>,
        fallback_desktop: ComIn<IVirtualDesktop>,
    ) -> HRESULT;

    pub unsafe fn find_desktop(
        &self,
        guid: *const GUID,
        out_desktop: *mut Option<IVirtualDesktop>,
    ) -> HRESULT {
        unsafe {
            IVirtualDesktopManagerInternal!(self, |inner| {
                inner.find_desktop(guid, out_desktop.forward())
            })
        }
    }

    pub unsafe fn get_desktop_switch_include_exclude_views(
        &self,
        desktop: ComIn<IVirtualDesktop>,
        out_pp_desktops1: *mut IObjectArray,
        out_pp_desktops2: *mut IObjectArray,
    ) -> HRESULT {
        match IVirtualDesktopManagerInternalInner::from_typed(self) {
            IVirtualDesktopManagerInternalInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopManagerInternalInner::build_22000(inner) => inner
                .get_desktop_switch_include_exclude_views(
                    desktop.forward(),
                    out_pp_desktops1,
                    out_pp_desktops2,
                ),
        }
    }

    pub unsafe fn set_name(&self, desktop: ComIn<IVirtualDesktop>, name: HSTRING) -> HRESULT {
        match IVirtualDesktopManagerInternalInner::from_typed(self) {
            IVirtualDesktopManagerInternalInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopManagerInternalInner::build_22000(inner) => {
                inner.set_name(desktop.forward(), name)
            }
        }
    }
    pub unsafe fn set_wallpaper(&self, desktop: ComIn<IVirtualDesktop>, name: HSTRING) -> HRESULT {
        match IVirtualDesktopManagerInternalInner::from_typed(self) {
            IVirtualDesktopManagerInternalInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopManagerInternalInner::build_22000(inner) => {
                inner.set_wallpaper(desktop.forward(), name)
            }
        }
    }
    pub unsafe fn update_wallpaper_for_all(&self, name: HSTRING) -> HRESULT {
        match IVirtualDesktopManagerInternalInner::from_typed(self) {
            IVirtualDesktopManagerInternalInner::build_10240(_) => E_NOTIMPL,
            IVirtualDesktopManagerInternalInner::build_22000(inner) => {
                inner.update_wallpaper_for_all(name)
            }
        }
    }
}
impl IVirtualDesktopManagerInternal {
    pub unsafe fn query_service(provider: &IServiceProvider) -> crate::Result<Self> {
        let mut obj = std::ptr::null_mut::<c_void>();
        unsafe {
            provider
                .query_service(
                    &CLSID_VirtualDesktopManagerInternal,
                    &IVirtualDesktopManagerInternal::IID(),
                    &mut obj,
                )
                .as_result()?;
        }
        assert_eq!(obj.is_null(), false);
        unsafe { Ok(IVirtualDesktopManagerInternal::from_raw(obj)) }
    }
}

support_interface!(as IVirtualDesktopPinnedAppsInner for IVirtualDesktopPinnedApps in all [build_10240, build_22000]);

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct IVirtualDesktopPinnedApps(IUnknown);

#[apply(forward_call)]
impl IVirtualDesktopPinnedApps {
    pub unsafe fn is_app_pinned(&self, app_id: PCWSTR, out_iss: *mut bool) -> HRESULT;
    pub unsafe fn pin_app(&self, app_id: PCWSTR) -> HRESULT;
    pub unsafe fn unpin_app(&self, app_id: PCWSTR) -> HRESULT;

    pub unsafe fn is_view_pinned(
        &self,
        view: ComIn<IApplicationView>,
        out_iss: *mut bool,
    ) -> HRESULT {
        unsafe {
            IVirtualDesktopPinnedApps!(self, |inner| inner.is_view_pinned(view.into(), out_iss))
        }
    }
    pub unsafe fn pin_view(&self, view: ComIn<IApplicationView>) -> HRESULT;
    pub unsafe fn unpin_view(&self, view: ComIn<IApplicationView>) -> HRESULT;
}
impl IVirtualDesktopPinnedApps {
    pub unsafe fn query_service(provider: &IServiceProvider) -> crate::Result<Self> {
        let mut obj = std::ptr::null_mut::<c_void>();
        unsafe {
            provider
                .query_service(
                    &CLSID_VirtualDesktopPinnedApps,
                    &IVirtualDesktopPinnedApps::IID(),
                    &mut obj,
                )
                .as_result()?;
        }
        assert_eq!(obj.is_null(), false);
        unsafe { Ok(IVirtualDesktopPinnedApps::from_raw(obj)) }
    }
}

// Bellow are helper methods that accesses the real COM interfaces. We could
// avoid the need for these helper methods by working with the IUnknown
// interface or implementing ComInterface for our abstraction types with the
// `IUnknown` IDD, even if that might get confusing.

struct IObjectArrayGetAtCallback<'a, T>(&'a IObjectArray, UINT, PhantomData<T>);
impl<COM, T> WithVersionedTypeCallback<COM, Result<T, windows::core::Error>>
    for IObjectArrayGetAtCallback<'_, T>
where
    // The COM interface for this specific Windows version:
    COM: ComInterface,
    // Should be possible to convert it into the more generic type:
    T: From<COM>,
{
    fn call(self) -> Result<T, windows::core::Error> {
        let com: COM = unsafe { self.0.GetAt::<COM>(self.1)? };
        Ok(From::from(com))
    }
}

/// Same as `GetAt` for `IObjectArray` but works even when we don't know the IID
/// of a COM interface at compile time.
#[allow(non_snake_case, private_bounds)]
pub unsafe fn IObjectArrayGetAt<'a, T>(
    object_array: &'a IObjectArray,
    index: UINT,
) -> Result<T, windows::core::Error>
where
    T: WithVersionedType<IObjectArrayGetAtCallback<'a, T>, Result<T, windows::core::Error>>,
{
    T::with_versioned_type(IObjectArrayGetAtCallback(object_array, index, PhantomData))
        .ok_or_else(|| windows::core::Error::from(E_NOTIMPL))?
}
