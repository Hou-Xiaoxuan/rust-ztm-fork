/// a test demo for pipy
use libc::{c_char, c_int};
use std::ffi::CString;
use std::thread;

mod util;

#[link(name = "pipy", kind = "dylib")]
extern "C" {
    pub fn pipy_main(argc: c_int, argv: *const *const c_char) -> c_int;

    pub fn pipy_exit(force: c_int);
}

pub fn start_agent(database: &str, listen_port: u16) {
    // run pipy repo://ztm/agent --database=database --listen=0.0.0.0:listen_port
    let database = database.to_string();
    tracing::info!("start pipy with port: {}", listen_port);
    let args = vec![
        CString::new("ztm-pipy").unwrap(),
        CString::new("repo://ztm/agent").unwrap(),
        CString::new("--args").unwrap(),
        CString::new(format!("--database={}", database)).unwrap(),
        CString::new(format!("--listen=0.0.0.0:{}", listen_port)).unwrap(),
    ];
    let c_args: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
    unsafe {
        pipy_main(c_args.len() as c_int, c_args.as_ptr());
    }
    thread::sleep(std::time::Duration::from_secs(1)); // wait for pipy to start
}

pub fn exit_agent() {
    unsafe {
        pipy_exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_start_agent() {
        util::init_logger("info");
        let port = 7776;
        start_agent("test.db", port);
        thread::sleep(std::time::Duration::from_secs(1));

        let resp = reqwest::get(format!("http://127.0.0.1:{}/api/version", port))
            .await
            .unwrap();
        tracing::debug!("resp: {:?}", resp);
        assert!(resp.status().is_success());
        tracing::info!("ztm agent start success");

        exit_agent();
        let resp = reqwest::get(format!("http://127.0.0.1:{}/api/version", port))
            .await
            .unwrap();
        tracing::debug!("resp: {:?}", resp);
        assert!(resp.status().as_u16() == 502); // 502
        tracing::info!("ztm agent exit success");
    }

    #[tokio::test]
    #[should_panic]
    /// didn't support multiple agent
    async fn test_start_multiple_agent() {
        util::init_logger("info");

        let port1 = 7777;
        let port2 = 7778;
        start_agent("test1.db", port1);

        let resp = reqwest::get(format!("http://0.0.0.0:{}/api/version", port1))
            .await
            .unwrap();
        assert!(resp.status().is_success());

        start_agent("test2.db", port2);
        let resp = reqwest::get(format!("http://0.0.0.0:{}/api/version", port2))
            .await
            .unwrap();
        assert!(resp.status().is_success());

        exit_agent();

        let resp = reqwest::get(format!("http://http://0.0.0.0:{}/api/version", port1))
            .await
            .unwrap();
        assert!(resp.status().as_u16() == 502); // 502
    }
}
