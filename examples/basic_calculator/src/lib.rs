#![no_std]

use core::panic::PanicInfo;

// Обработчик паники для no_std среды
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Простая функция сложения двух u64 чисел
/// Экспортируется для вызова из WebAssembly
#[no_mangle]
pub extern "C" fn add_u64(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

/// Простая функция умножения двух u64 чисел
#[no_mangle]
pub extern "C" fn mul_u64(a: u64, b: u64) -> u64 {
    a.wrapping_mul(b)
}

/// Сложение с проверкой на переполнение (возвращает 0 при переполнении)
#[no_mangle]
pub extern "C" fn add_safe(a: u64, b: u64) -> u64 {
    match a.checked_add(b) {
        Some(result) => result,
        None => 0,
    }
}

/// Умножение с проверкой на переполнение
#[no_mangle]
pub extern "C" fn mul_safe(a: u64, b: u64) -> u64 {
    match a.checked_mul(b) {
        Some(result) => result,
        None => 0,
    }
}

/// Функция для получения версии модуля
#[no_mangle]
pub extern "C" fn get_version() -> u32 {
    0x00010000 // Версия 1.0.0 в формате MAJOR.MINOR.PATCH
}
