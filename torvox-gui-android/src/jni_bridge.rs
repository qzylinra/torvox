// @JNI bridge for NDK functions, IMPL_ANDR_003, impl, [REQ_ANDR_003]
// @need-ids: REQ_ANDR_003
//! JNI bridge for calling NDK functions that boltffi/JNA cannot provide.
//! Provides a reliable way to obtain an ANativeWindow pointer from a Surface object.

use core::ffi::c_void;

// JNI opaque types
type JNIEnvPtr = *mut c_void;
type JClassPtr = *mut c_void;
type JObjectPtr = *mut c_void;

#[cfg(target_os = "android")]
#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_fromSurface(env: JNIEnvPtr, surface: JObjectPtr) -> *mut c_void;
}

/// JNI export: io.torvox.bridge.NativeWindow.getNativeWindowPtr(Surface) -> Long
/// Returns ANativeWindow pointer as jlong (i64).
/// This is the NDK-recommended way to obtain ANativeWindow from a Surface.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "system" fn Java_io_torvox_bridge_NativeWindow_getNativeWindowPtr(
    env: JNIEnvPtr,
    _class: JClassPtr,
    surface: JObjectPtr,
) -> i64 {
    let ptr = unsafe { ANativeWindow_fromSurface(env, surface) };
    log::debug!("JNI ANativeWindow_fromSurface returned: {:p}", ptr);
    ptr as i64
}
