use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use wland_client::ThreadPool;
use std::io;
use std::io::Bytes;

fn main() -> Result<(), io::Error> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    loop {
        let pool = ThreadPool::new(100);
        for stream in listener.incoming().take(10)
        {
            let stream = stream?;
            pool.execute(|| {
                handle_connection(stream
                                  , "hu60.cn"
                                  , "http"
                                  , 80);
            });
        }
    }

    Ok(())
}

const HAS_READ: usize = 1024;

fn handle_connection(mut cstream: TcpStream, host: &str, http_dup: &str, port: usize) -> Result<(), io::Error> {
    /**
    * get `client` request header
    */
    let header = {
        let mut buff = [0; HAS_READ];
        let mut size = HAS_READ;
        let mut header = String::new();
        while size >= HAS_READ {
            size = cstream.read(&mut buff[0..size])?;
            header.push_str(String::from_utf8_lossy(&buff).as_ref());
        }
        header.push_str(String::from_utf8_lossy(&buff[..size]).as_ref());
        header
    };
    /**
    * DIY the `client` header
    */
    let dheader = {
        let mut dh = String::new();
        for line in header.lines() {
            let mut line = line.trim().to_string();
            let l = line.trim().to_lowercase();
            if l.starts_with("host") {
                line = format!("Host: {}", host);
            } else if l.starts_with("referer") {
                line = format!("Referer: {}://{}", http_dup, host);
            }
            dh.push_str(&line);
            dh.push_str("\r\n");
        }
        // println!("{}", dh);
        dh
    };
    /**
    * request forward to destination connection
    */
    {
        // destination connection
        let mut dstream = TcpStream::connect(format!("{}:{}", &host, &port))?;
        dstream.write(dheader.as_bytes());

        // read from server
        let mut buff = [0; HAS_READ];
        let mut size = HAS_READ;
        while size >= HAS_READ {
            // read from destination
            size = dstream.read(&mut buff)?;
            // write to client
            cstream.write(&buff[..size]);
        }
        cstream.write(&buff[..size]);
        cstream.flush()?;
    }

    Ok(())
}
