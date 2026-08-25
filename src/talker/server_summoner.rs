pub fn server_summoner() {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("[-] Failed to build Tokio runtime: {:?}", e);
            return;
        }
    };

    rt.block_on(async {
        if let Err(err) = super::talker::start_server().await {
            eprintln!("[-] Axum server error: {:?}", err);
        }
    });
}