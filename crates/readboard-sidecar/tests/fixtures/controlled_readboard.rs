use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::Path,
    thread,
    time::Duration,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    assert_eq!(&args[..6], &["yzy", " ", " ", " ", "1", "cn"]);
    assert_eq!(args.len(), 7);
    let exe = env::current_exe().unwrap();
    let mode = exe.file_stem().unwrap().to_str().unwrap();
    fs::write(exe.with_extension("pid"), std::process::id().to_string()).unwrap();
    if mode == "early_exit" {
        std::process::exit(23);
    }
    if mode == "no_connect" {
        thread::sleep(Duration::from_secs(30));
        return;
    }
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", args[6])).unwrap();
    fs::write(exe.with_extension("connected"), &args[6]).unwrap();
    if mode == "tcp_only" {
        thread::sleep(Duration::from_secs(30));
        return;
    }
    if mode == "late_ready" {
        while !Path::new(&exe.with_extension("release")).exists() {
            thread::sleep(Duration::from_millis(5));
        }
    }
    if mode == "version_first" {
        stream.write_all(b"version: 220430\n").unwrap();
    }
    if mode == "split" {
        stream.write_all(b"rea").unwrap();
        thread::sleep(Duration::from_millis(15));
        stream.write_all(b"dy\r\n").unwrap();
    } else {
        let _ = stream.write_all(b"ready\n");
    }
    let mut input = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if input.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    if line != "version\n" {
        return;
    }
    fs::write(exe.with_extension("requested"), "version").unwrap();
    match mode {
        "wrong" => stream.write_all(b"version: 220429\n").unwrap(),
        "future" => stream.write_all(b"version: 999999\n").unwrap(),
        "empty_version" => stream.write_all(b"version: \n").unwrap(),
        "missing_version" | "version_first" => {}
        "oversized" => stream.write_all(&vec![b'x'; 9000]).unwrap(),
        "split" => {
            stream.write_all(b"version: 220").unwrap();
            thread::sleep(Duration::from_millis(15));
            stream.write_all(b"430\r\nbothSync\n").unwrap();
        }
        _ => {
            let _ = stream.write_all(b"version: 220430\n");
        }
    }
    if mode == "disconnect" {
        stream.shutdown(std::net::Shutdown::Both).unwrap();
        thread::sleep(Duration::from_secs(30));
        return;
    }
    if mode == "ignore_quit" {
        thread::sleep(Duration::from_secs(30));
        return;
    }
    if mode == "multi_frames" {
        stream.write_all(b"start 3 3\n").unwrap();
        // Batch 1: split across writes
        stream.write_all(b"syncPlatform fox\nroomToken room-1\n").unwrap();
        thread::sleep(Duration::from_millis(15));
        stream.write_all(b"liveTitleMove 5\nre=0,0,0\n").unwrap();
        thread::sleep(Duration::from_millis(15));
        stream.write_all(b"re=0,1,0\nre=0,0,2\nend\n").unwrap();
        thread::sleep(Duration::from_millis(15));

        // Batch 2: malformed batch (digit 9 is invalid; and only 2 rows for 3x3 board)
        stream
            .write_all(b"syncPlatform fox\nre=0,0,0\nre=0,9,0\nend\n")
            .unwrap();
        thread::sleep(Duration::from_millis(15));

        // Batch 3: coalesced in one write
        stream
            .write_all(
                b"syncPlatform fox\nroomToken room-2\nliveTitleMove 7\nre=1,0,0\nre=0,2,0\nre=0,0,3\nend\n",
            )
            .unwrap();
    }
    if mode == "frames_disconnect" {
        stream
            .write_all(
                b"start 3 3\nsyncPlatform fox\nroomToken room-1\nliveTitleMove 5\nre=0,0,0\nre=0,1,0\nre=0,0,2\nend\n",
            )
            .unwrap();
        thread::sleep(Duration::from_millis(30));
        stream.shutdown(std::net::Shutdown::Both).unwrap();
        thread::sleep(Duration::from_secs(30));
        return;
    }
    if mode == "late_frames" {
        stream
            .write_all(
                b"start 3 3\nsyncPlatform fox\nroomToken room-early\nliveTitleMove 1\nre=0,0,0\nre=0,1,0\nre=0,0,2\nend\n",
            )
            .unwrap();
        while !Path::new(&exe.with_extension("release")).exists() {
            thread::sleep(Duration::from_millis(5));
        }
        let _ =
            stream.write_all(b"roomToken late-line\nliveTitleMove 99\nre=1,1,1\nre=2,2,2\nre=0,0,0\nend\n");
    }
    loop {
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) if line == "loss\n" => {
                let mut loss_file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(exe.with_extension("loss"))
                    .unwrap();
                loss_file.write_all(b"loss\n").unwrap();
            }
            Ok(_) if line == "quit\n" => {
                fs::write(exe.with_extension("quit"), "quit").unwrap();
                return;
            }
            _ => {}
        }
    }
}
