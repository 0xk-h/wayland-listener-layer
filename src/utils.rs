use std::process::Command;

/// Create an anonymous Unix pipe, returning (read_fd, write_fd)
pub fn nix_pipe() -> (i32, i32) {
    let mut fds = [0i32; 2];
    unsafe {
        libc::pipe(fds.as_mut_ptr());
    }
    (fds[0], fds[1])
}

/// Decode %XX percent-encoding in URI paths safely
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push(h << 4 | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// final output so pipe the output to whichever u need
pub fn output(path: String) {
    let filename = path.rsplit("/").next().unwrap();

    let status = Command::new("notify-send")
        .arg(filename)
        .arg(path)
        .status()
        .expect("failed to pipe the output");

    print!("Exit status: {}", status);
}