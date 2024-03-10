#[allow(dead_code)]
extern "C" {
    fn user_handle();
    fn user_return();
}

#[no_mangle]
pub fn user_trap() {

    unsafe { user_return(); }
}
