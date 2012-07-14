import libc::c_int;

#[link_args = "c_signal.o"]
#[nolink]
extern mod c_signal {
    fn signal_init();
    fn signal_received() -> c_int;
}

fn init() {
    c_signal::signal_init();
}

fn signal_received() -> bool {
    ret c_signal::signal_received() as bool;
}
