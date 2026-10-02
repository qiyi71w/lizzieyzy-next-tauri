use std::io::{self, BufRead, BufReader, Read};
use std::process::{ChildStderr, ChildStdout};
use std::sync::mpsc::{self, Receiver};
use std::thread;

const MAX_LINE_BYTES: usize = 64 * 1024;
const MAX_QUEUED_LINES: usize = 64;
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_RESPONSE_LINES: usize = 1024;
const MAX_STDERR_BYTES: usize = 8 * 1024;

pub(crate) fn spawn_stdout_reader(stdout: ChildStdout) -> Receiver<io::Result<Option<String>>> {
    let (sender, receiver) = mpsc::sync_channel(MAX_QUEUED_LINES);
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let line = read_stdout_line(&mut reader);
            let finished = !matches!(&line, Ok(Some(_)));
            if sender.send(line).is_err() || finished {
                break;
            }
        }
    });
    receiver
}

fn read_stdout_line(reader: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut line = Vec::new();
    loop {
        let available = match reader.fill_buf() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "GTP stdout ended in an incomplete line",
                ))
            };
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let length = newline.unwrap_or(available.len());
        // The limit includes a possible CR before LF, but not LF itself.
        if length > MAX_LINE_BYTES - line.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GTP stdout line exceeds 64 KiB",
            ));
        }
        let required = line.len() + length;
        if required > line.capacity() {
            let capacity = (line.capacity() * 2).max(required).min(MAX_LINE_BYTES);
            line.reserve_exact(capacity - line.len());
        }
        line.extend_from_slice(&available[..length]);
        reader.consume(length + usize::from(newline.is_some()));
        if newline.is_some() {
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return String::from_utf8(line)
                .map(Some)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "GTP stdout is not valid UTF-8"));
        }
    }
}

pub(crate) fn spawn_stderr_reader(mut stderr: ChildStderr) -> Receiver<io::Result<String>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut retained = Vec::with_capacity(MAX_STDERR_BYTES);
        let mut buffer = [0; 4096];
        loop {
            match stderr.read(&mut buffer) {
                Ok(0) => {
                    let _ = sender.send(Ok(stderr_summary(&retained)));
                    break;
                }
                Ok(length) => {
                    let keep = length.min(MAX_STDERR_BYTES - retained.len());
                    retained.extend_from_slice(&buffer[..keep]);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
    });
    receiver
}

fn stderr_summary(mut bytes: &[u8]) -> String {
    let mut summary = String::with_capacity((bytes.len() * 3).min(MAX_STDERR_BYTES));
    while !bytes.is_empty() {
        let (valid, invalid) = match std::str::from_utf8(bytes) {
            Ok(text) => (text, 0),
            Err(error) => (
                // The prefix ending at valid_up_to is guaranteed valid UTF-8.
                std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap(),
                error.error_len().unwrap_or(bytes.len() - error.valid_up_to()),
            ),
        };
        let mut keep = valid.len().min(MAX_STDERR_BYTES - summary.len());
        while !valid.is_char_boundary(keep) {
            keep -= 1;
        }
        summary.push_str(&valid[..keep]);
        if keep < valid.len() || invalid == 0 {
            break;
        }
        if MAX_STDERR_BYTES - summary.len() < '\u{fffd}'.len_utf8() {
            break;
        }
        summary.push('\u{fffd}');
        bytes = &bytes[valid.len() + invalid..];
    }
    summary
}

pub(crate) struct GtpResponse {
    pub(crate) success: bool,
    pub(crate) body: String,
}

pub(crate) struct ResponseDecoder {
    expected_id: String,
    success: Option<bool>,
    body: String,
    response_bytes: usize,
    response_lines: usize,
    complete: bool,
}

impl ResponseDecoder {
    pub(crate) fn new(expected_id: u32) -> Self {
        Self {
            expected_id: expected_id.to_string(),
            success: None,
            body: String::new(),
            response_bytes: 0,
            response_lines: 0,
            complete: false,
        }
    }

    pub(crate) fn push(&mut self, line: &str) -> Result<Option<GtpResponse>, String> {
        if self.complete {
            return Err("GTP response already completed".to_owned());
        }
        // Count normalized wire bytes and lines, including the final delimiter.
        if line.len() >= MAX_RESPONSE_BYTES - self.response_bytes {
            return Err("GTP response exceeds 64 KiB".to_owned());
        }
        if self.response_lines == MAX_RESPONSE_LINES {
            return Err("GTP response exceeds 1024 lines".to_owned());
        }
        self.response_bytes += line.len() + 1;
        self.response_lines += 1;
        if line
            .chars()
            .any(|character| character.is_control() && character != '\t')
        {
            return Err("GTP response contains a control character".to_owned());
        }
        if let Some(success) = self.success {
            if line.is_empty() {
                self.complete = true;
                return Ok(Some(GtpResponse {
                    success,
                    body: std::mem::take(&mut self.body),
                }));
            }
            if line.starts_with(['=', '?']) {
                return Err("GTP response contains a second header before its delimiter".to_owned());
            }
            self.body.push('\n');
            self.body.push_str(line);
            return Ok(None);
        }
        let Some(marker) = line.as_bytes().first() else {
            return Err("GTP response starts with an unexpected empty line".to_owned());
        };
        if !matches!(marker, b'=' | b'?') {
            return Err("GTP response header must start with = or ?".to_owned());
        }
        let after_marker = &line[1..];
        let id_length = after_marker.bytes().take_while(u8::is_ascii_digit).count();
        if id_length == 0 {
            return Err("GTP response header is missing its numeric command ID".to_owned());
        }
        if after_marker[..id_length] != self.expected_id {
            return Err(format!(
                "GTP response command ID does not match expected {}",
                self.expected_id
            ));
        }
        let after_id = &after_marker[id_length..];
        let body = if after_id.is_empty() {
            ""
        } else if after_id.starts_with([' ', '\t']) {
            &after_id[1..]
        } else {
            return Err("GTP response command ID must be followed by space, tab, or end of line".to_owned());
        };
        self.success = Some(*marker == b'=');
        self.body.push_str(body);
        Ok(None)
    }
}
