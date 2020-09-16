use std::fs;
use std::io;
use std::io::Bytes;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use wland_client::ThreadPool;

fn main() -> Result<(), io::Error> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(10);
    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(|| {
            if let Ok(_) = handle_connection(stream
                                             , "127.0.0.1"
                                             , "http"
                                             , 8081) {}
        });
    }

    Ok(())
}

const HAS_READ: usize = 1024;

fn handle_connection(mut cstream: TcpStream, host: &str, http_dup: &str, port: usize) -> Result<(), io::Error> {
    /**
    * get `client` request header
    */
    let cdata = {
        let mut buff = [0; HAS_READ];
        let mut size = HAS_READ;
        let mut col = Vec::new();
        while size == HAS_READ {
            size = cstream.read(&mut buff)?;
            col.append(&mut buff[..size].to_vec());
        }
        col
    };

    /**
     * get ci
     */
    let ci = {
        let mut ci = 0;
        let mut i = 0;
        for c in &cdata {
            if c.eq(&13) && cdata[i + 1].eq(&10)
                && cdata[i + 2].eq(&13) && cdata[i + 3].eq(&10) {
                ci = i + 4;
                break;
            }
            i += 1;
        }
        // 无双\r\n，为无效请求头
        if ci == 0 {
            return Ok(());
        } else {
            ci
        }
    };
    /**
     * DIY the `client` header
     */
    let cheader = String::from_utf8_lossy(&cdata[..ci]);
    let dheader = {
        let mut dh = String::new();
        for line in cheader.lines() {
            let mut line = line.trim().to_string();
            let l = line.trim().to_lowercase();
            if l.starts_with("host") {
                line = format!("Host: {}:{}", host, port);
            } else if l.starts_with("referer") {
                line = format!("Referer: {}://{}:{}/", http_dup, host, port);
            }
            dh.push_str(&line);
            dh.push_str("\r\n");
        }
        println!("{}", dh);
        dh
    };
    /**
     * request forward to destination connection
     */
    let cdata = &cdata[ci..];
    {
        // destination connection
        let mut dstream = {
            if dheader.contains("/user") {
                TcpStream::connect(format!("{}:{}", &host, 8080))?
            } else {
                TcpStream::connect(format!("{}:{}", &host, &port))?
            }
        };
        dstream.write(&dheader.as_bytes())?;
        dstream.write(&cdata)?;
        dstream.flush()?;

        // read from server
        let mut buff = [0; HAS_READ];
        let mut size = HAS_READ;
        while size == HAS_READ {
            // read from destination
            size = dstream.read(&mut buff)?;
            // write to client
            cstream.write(&buff[..size])?;
        }
        cstream.flush()?;
    }

    Ok(())
}
