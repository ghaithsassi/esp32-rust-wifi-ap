use defmt::info;
use embassy_net::tcp::TcpSocket;
use embassy_net::IpListenEndpoint;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};

pub async fn start_server(stack: Stack<'static>, gw_ip_addr_str: &'static str) {
    info!(
        "Connect to the AP `esp-radio` and point your browser to http://{:?}:8080/",
        gw_ip_addr_str
    );
    info!("DHCP is enabled so there's no need to configure a static IP, just in case:");

    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }
    stack.config_v4().inspect(|_| info!("ipv4 config:"));

    let mut rx_buffer = [0; 1536];
    let mut tx_buffer = [0; 1536];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    loop {
        info!("Wait for connection...");
        let r = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await;
        info!("Connected...");

        if let Err(e) = r {
            info!("connect error: {:?}", e);
            continue;
        }

        use embedded_io_async::Write;

        let mut buffer = [0u8; 1024];
        let mut pos = 0;
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    info!("read EOF");
                    break;
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };

                    if to_print.contains("\r\n\r\n") {
                        info!("{}", to_print);
                        break;
                    }

                    pos += len;
                }
                Err(e) => {
                    info!("read error: {:?}", e);
                    break;
                }
            };
        }

        let r = socket
            .write(
                b"HTTP/1.0 200 OK\r\n\r\n\
            <html>\
                <body>\
                    <h1>Hello Rust! Hello esp-radio!</h1>\
                </body>\
            </html>\r\n\
            ",
            )
            .await;
        if let Err(e) = r {
            info!("write error: {:?}", e);
        }

        let r = socket.flush().await;
        if let Err(e) = r {
            info!("flush error: {:?}", e);
        }
        Timer::after(Duration::from_millis(1000)).await;

        socket.close();
        Timer::after(Duration::from_millis(1000)).await;

        socket.abort();
    }
}
