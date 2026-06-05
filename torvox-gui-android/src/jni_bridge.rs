//! JNI 桥接，用于调用 boltffi/JNA 无法提供的 NDK 函数。
//! 提供从 Surface 对象获取 ANativeWindow 指针的可靠方式。

use core::ffi::c_void;

// JNI 不透明类型
type JNIEnvPtr = *mut c_void;
type JClassPtr = *mut c_void;
type JObjectPtr = *mut c_void;

#[cfg(target_os = "android")]
#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_fromSurface(env: JNIEnvPtr, surface: JObjectPtr) -> *mut c_void;
    #[allow(dead_code)]
    fn ANativeWindow_release(window: *mut c_void);
}

/// JNI 导出: io.torvox.bridge.NativeWindow.getNativeWindowPtr(Surface) -> Long
/// 返回 ANativeWindow 指针作为 jlong (i64)。
/// 这是 NDK 推荐的从 Surface 获取 ANativeWindow 的方式。
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_torvox_bridge_NativeWindow_getNativeWindowPtr(
    env: JNIEnvPtr,
    _class: JClassPtr,
    surface: JObjectPtr,
) -> i64 {
    let ptr = unsafe { ANativeWindow_fromSurface(env, surface) };
    log::debug!("JNI ANativeWindow_fromSurface returned: {:p}", ptr);
    ptr as i64
}
