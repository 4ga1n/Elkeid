use crossbeam_channel::bounded;
use log::*;
use scanner::{
    config::{
        CACHE_SIZE, NAME, RULES_SET, SCAN_DIR_CONFIG, SOCKET_PATH, VERSION, WAIT_INTERVAL_DAILY,
    },
    cronjob::Cronjob,
    detector::Detector,
    fmonitor::FileMonitor,
};

use std::{thread, time::Duration};

fn main() {
    let pid = std::process::id();
    println!("scanner pid : {:?}", pid);

    let builder = plugin_builder::Builder::new(SOCKET_PATH, NAME, VERSION).unwrap();
    let (sender, receiver) = builder.set_name(NAME.to_string()).build();
    info!("Elkeid-scanner Init success!!!");

    let (s, r) = bounded(20);
    let (s_lock, r_lock) = bounded(0);

    // main scanner worker
    let s_recv_worker = s.clone();
    let s_recv_lock = s_lock.clone();
    let mut mworker = Detector::new(
        sender,
        receiver,
        s_recv_worker,
        r,
        s_recv_lock,
        RULES_SET,
        CACHE_SIZE,
    );
    thread::spawn(move || loop {
        let mut _timeout = 300;
        mworker.work(Duration::from_secs(_timeout));
    });

    // cronjob scan dir and proc
    let s_cron_worker = s.clone();
    let s_cron_lock = s_lock.clone();
    let _cronjob_t = Cronjob::new(s_cron_worker, s_cron_lock, WAIT_INTERVAL_DAILY);

    // file_monitor scan
    let s_fano_worker = s.clone();
    let s_fano_lock = s_lock.clone();
    let mut fmonitor_t = FileMonitor::new(s_fano_worker, s_fano_lock).unwrap();
    for each in SCAN_DIR_CONFIG {
        if let Err(e) = fmonitor_t.add(each.fpath) {
            error!("fmonitor add fpath err {:?}:{:?}", each.fpath, e);
        }
    }

    // wait childs
    let _: () = r_lock.recv().unwrap();
    info!("[Main exit] loop outside !!!");
}
