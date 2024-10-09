use super::manager;

pub fn init() {
    let tcb = manager::create_tcb_by_app_name("init").expect("must load");

    manager::push_to_runq(tcb);
}
