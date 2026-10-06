use go_core::{
    ReadboardControl, ReadboardFrame, ReadboardLastMoveSource, ReadboardPlatform, ReadboardRemoteContext,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadboardInbound {
    Frame(ReadboardFrame),
    Control(ReadboardControl),
    Rejected(String),
}

#[derive(Debug, Default)]
pub struct FrameDecoder {
    board_size: Option<(u8, u8)>,
    pending_context: ReadboardRemoteContext,
    pending_rows: Vec<String>,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn receive(&mut self, line: &str) -> Option<ReadboardInbound> {
        let line = line.trim_end_matches('\r');
        let trimmed = line.trim();

        if trimmed.starts_with("endsync") {
            self.reset_pending();
            return Some(ReadboardInbound::Control(ReadboardControl::EndSync));
        }

        if trimmed.starts_with("stopsync") {
            self.reset_pending();
            return Some(ReadboardInbound::Control(ReadboardControl::StopSync));
        }

        if trimmed == "clearBoard" {
            return Some(ReadboardInbound::Control(ReadboardControl::ClearBoard));
        }

        if trimmed == "clear" {
            self.reset_pending();
            return Some(ReadboardInbound::Control(ReadboardControl::Clear));
        }

        if trimmed == "sync" {
            return Some(ReadboardInbound::Control(ReadboardControl::Sync));
        }

        let is_start = trimmed == "start"
            || trimmed
                .strip_prefix("start")
                .is_some_and(|rest| rest.starts_with(' ') || rest.starts_with('\t'));

        if is_start {
            let mut parts = trimmed.split_whitespace();
            let _ = parts.next(); // consumes "start"
            let w_opt = parts.next().and_then(|s| s.parse::<u8>().ok());
            let h_opt = parts.next().and_then(|s| s.parse::<u8>().ok());
            let size = match (w_opt, h_opt) {
                (Some(w), Some(h)) if (2..=25).contains(&w) && (2..=25).contains(&h) => Some((w, h)),
                _ => None,
            };
            if let Some(s) = size {
                self.board_size = Some(s);
            }
            self.reset_pending();
            return Some(ReadboardInbound::Control(ReadboardControl::Start { size }));
        }

        if let Some(rest) = trimmed.strip_prefix("syncPlatform ") {
            let platform = ReadboardPlatform::parse(rest);
            self.fold(|context| context.with_platform(platform));
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("roomToken ") {
            self.fold(|context| context.with_room_token(rest.trim()));
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("liveTitleMove ") {
            let move_num = rest.trim().parse::<u32>().ok();
            self.fold(|context| context.with_live_title_move(move_num));
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("recordCurrentMove ") {
            let move_num = rest.trim().parse::<u32>().ok();
            self.fold(|context| context.with_record_current_move(move_num));
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("recordTotalMove ") {
            let move_num = rest.trim().parse::<u32>().ok();
            self.fold(|context| context.with_record_total_move(move_num));
            return None;
        }

        if trimmed.starts_with("recordAtEnd ") {
            let at_end = trimmed.ends_with('1');
            self.fold(|context| context.with_record_at_end(at_end));
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("recordTitleFingerprint ") {
            self.fold(|context| context.with_title_fingerprint(rest.trim()));
            return None;
        }

        if trimmed == "forceRebuild" {
            self.fold(|context| context.with_force_rebuild(true));
            return None;
        }

        if trimmed.starts_with("foxMoveNumber") {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.len() == 2 && tokens[0] == "foxMoveNumber" {
                if let Ok(num) = tokens[1].parse::<u32>() {
                    self.fold(|context| context.with_fox_move_number(Some(num)));
                }
            }
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("lastMoveSource ") {
            let source = ReadboardLastMoveSource::parse(rest.trim());
            self.fold(|context| context.with_last_move_source(source));
            return None;
        }

        if let Some(payload) = trimmed.strip_prefix("re=") {
            self.pending_rows.push(payload.to_string());
            return None;
        }

        if trimmed == "end" {
            let context =
                std::mem::replace(&mut self.pending_context, ReadboardRemoteContext::generic(false));
            let rows = std::mem::take(&mut self.pending_rows);

            let (width, height) = match self.board_size {
                Some(size) => size,
                None => {
                    return Some(ReadboardInbound::Rejected(
                        "missing board size (no prior start)".to_string(),
                    ));
                }
            };

            if rows.len() != height as usize {
                return Some(ReadboardInbound::Rejected(format!(
                    "expected {height} rows, got {}",
                    rows.len()
                )));
            }

            let mut codes = Vec::with_capacity((width as usize) * (height as usize));
            for row in rows {
                let tokens: Vec<&str> = row.split(',').collect();
                if tokens.len() != width as usize {
                    return Some(ReadboardInbound::Rejected(format!(
                        "expected {width} tokens in row, got {}",
                        tokens.len()
                    )));
                }
                for token in tokens {
                    match parse_code(token) {
                        Some(code) => codes.push(code),
                        None => {
                            return Some(ReadboardInbound::Rejected(format!(
                                "invalid cell code '{token}': expected single ASCII digit 0..=4"
                            )));
                        }
                    }
                }
            }

            return Some(ReadboardInbound::Frame(ReadboardFrame {
                width,
                height,
                codes,
                context,
            }));
        }

        None
    }

    fn fold(&mut self, change: impl FnOnce(ReadboardRemoteContext) -> ReadboardRemoteContext) {
        self.pending_context = change(std::mem::take(&mut self.pending_context));
    }

    fn reset_pending(&mut self) {
        self.pending_context = ReadboardRemoteContext::generic(false);
        self.pending_rows.clear();
    }
}

fn parse_code(token: &str) -> Option<u8> {
    if token.len() == 1 && matches!(token.as_bytes()[0], b'0'..=b'4') {
        Some(token.as_bytes()[0] - b'0')
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use go_core::ReadboardWindowKind;

    #[test]
    fn test_full_fox_live_frame() {
        let mut decoder = FrameDecoder::new();
        assert_eq!(
            decoder.receive("start 19 19"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((19, 19))
            }))
        );
        assert_eq!(decoder.receive("syncPlatform fox"), None);
        assert_eq!(decoder.receive("roomToken 43581号"), None);
        assert_eq!(decoder.receive("liveTitleMove 42"), None);
        assert_eq!(decoder.receive("forceRebuild"), None);
        assert_eq!(decoder.receive("foxMoveNumber 42"), None);
        assert_eq!(decoder.receive("lastMoveSource redBlueMarker"), None);

        for y in 0..19 {
            let mut row = vec!["0"; 19];
            if y == 0 {
                row[0] = "1";
            } else if y == 1 {
                row[1] = "2";
            } else if y == 2 {
                row[2] = "3";
            } else if y == 3 {
                row[3] = "4";
            }
            let line = format!("re={}", row.join(","));
            assert_eq!(decoder.receive(&line), None);
        }

        let inbound = decoder.receive("end");
        let Some(ReadboardInbound::Frame(frame)) = inbound else {
            panic!("expected Frame, got {inbound:?}");
        };

        assert_eq!(frame.width, 19);
        assert_eq!(frame.height, 19);
        assert_eq!(frame.codes.len(), 361);
        assert_eq!(frame.codes[0], 1); // (0,0)
        assert_eq!(frame.codes[20], 2); // (1,1) = 1*19+1
        assert_eq!(frame.codes[40], 3); // (2,2)
        assert_eq!(frame.codes[60], 4); // (3,3)
        assert_eq!(frame.codes[1], 0); // (1,0)

        assert_eq!(frame.context.platform, ReadboardPlatform::Fox);
        assert_eq!(frame.context.window_kind, ReadboardWindowKind::LiveRoom);
        assert_eq!(frame.context.room_token.as_deref(), Some("43581号"));
        assert_eq!(frame.context.live_title_move, Some(42));
        assert!(frame.context.force_rebuild);
        assert_eq!(frame.context.fox_move_number, Some(42));
        assert_eq!(
            frame.context.last_move_source,
            ReadboardLastMoveSource::RedBlueMarker
        );
        assert_eq!(frame.context.record_current_move, None);
        assert_eq!(frame.context.record_total_move, None);
        assert!(!frame.context.record_at_end);
        assert_eq!(frame.context.title_fingerprint, None);
    }

    #[test]
    fn test_record_view_folding_and_order_sensitive_clearing() {
        let mut decoder = FrameDecoder::new();
        assert_eq!(
            decoder.receive("start 19 19"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((19, 19))
            }))
        );
        decoder.receive("syncPlatform fox");
        decoder.receive("recordTitleFingerprint record-fp-1");
        decoder.receive("recordCurrentMove 10");
        decoder.receive("recordTotalMove 50");
        decoder.receive("recordAtEnd 1");

        // Order-sensitive switch: roomToken after record lines clears record fields
        decoder.receive("roomToken 12345");

        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }

        let Some(ReadboardInbound::Frame(frame)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame.context.window_kind, ReadboardWindowKind::LiveRoom);
        assert_eq!(frame.context.room_token.as_deref(), Some("12345"));
        assert_eq!(frame.context.record_current_move, None);
        assert_eq!(frame.context.record_total_move, None);
        assert!(!frame.context.record_at_end);
        assert_eq!(frame.context.title_fingerprint, None);

        // Test invalid recordTotalMove clears it
        decoder.receive("syncPlatform fox");
        decoder.receive("recordTitleFingerprint fp-2");
        decoder.receive("recordTotalMove 50");
        decoder.receive("recordTotalMove invalid");
        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        let Some(ReadboardInbound::Frame(frame2)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame2.context.window_kind, ReadboardWindowKind::RecordView);
        assert_eq!(frame2.context.record_total_move, None);
        assert_eq!(frame2.context.title_fingerprint.as_deref(), Some("fp-2"));

        // Invalid recordCurrentMove and liveTitleMove clear to None
        decoder.receive("recordCurrentMove 15");
        decoder.receive("recordCurrentMove not_a_num");
        decoder.receive("liveTitleMove 25");
        decoder.receive("liveTitleMove bad");
        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        let Some(ReadboardInbound::Frame(frame3)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame3.context.live_title_move, None);
    }

    #[test]
    fn test_invalid_fox_move_number_leaves_previous_value() {
        let mut decoder = FrameDecoder::new();
        decoder.receive("start 19 19");
        decoder.receive("foxMoveNumber 42");
        decoder.receive("foxMoveNumber nope");
        decoder.receive("foxMoveNumber");
        decoder.receive("foxMoveNumber 1 2 3");
        decoder.receive("foxMoveNumber -5");

        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        let Some(ReadboardInbound::Frame(frame)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame.context.fox_move_number, Some(42));
    }

    #[test]
    fn test_missing_last_move_source_stays_legacy_unknown_and_unknown_token() {
        let mut decoder = FrameDecoder::new();
        decoder.receive("start 19 19");

        // Frame 1: missing lastMoveSource stays LegacyUnknown
        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        let Some(ReadboardInbound::Frame(frame1)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(
            frame1.context.last_move_source,
            ReadboardLastMoveSource::LegacyUnknown
        );

        // Frame 2: unknown token -> Unknown
        decoder.receive("lastMoveSource someWeirdToken");
        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        let Some(ReadboardInbound::Frame(frame2)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame2.context.last_move_source, ReadboardLastMoveSource::Unknown);

        // Frame 3: known tokens
        decoder.receive("lastMoveSource redBlueMarker");
        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        let Some(ReadboardInbound::Frame(frame3)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(
            frame3.context.last_move_source,
            ReadboardLastMoveSource::RedBlueMarker
        );
    }

    #[test]
    fn test_non_square_4x3_frame_index_mapping() {
        let mut decoder = FrameDecoder::new();
        assert_eq!(
            decoder.receive("start 4 3"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((4, 3))
            }))
        );
        decoder.receive("re=1,0,0,2");
        decoder.receive("re=0,3,4,0");
        decoder.receive("re=2,0,1,0");

        let Some(ReadboardInbound::Frame(frame)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame.width, 4);
        assert_eq!(frame.height, 3);
        assert_eq!(frame.codes.len(), 12);

        // codes[y * W + x]
        // y=0
        assert_eq!(frame.codes[0], 1);
        assert_eq!(frame.codes[1], 0);
        assert_eq!(frame.codes[2], 0);
        assert_eq!(frame.codes[3], 2);
        // y=1
        assert_eq!(frame.codes[4], 0);
        assert_eq!(frame.codes[5], 3);
        assert_eq!(frame.codes[6], 4);
        assert_eq!(frame.codes[7], 0);
        // y=2
        assert_eq!(frame.codes[8], 2);
        assert_eq!(frame.codes[9], 0);
        assert_eq!(frame.codes[10], 1);
        assert_eq!(frame.codes[11], 0);
    }

    #[test]
    fn test_rejected_cases_and_recovery() {
        let mut decoder = FrameDecoder::new();

        // 1. No prior start -> Rejected
        decoder.receive("re=0,0");
        decoder.receive("re=0,0");
        let res = decoder.receive("end");
        assert!(matches!(res, Some(ReadboardInbound::Rejected(_))));

        // Valid start then valid frame recovers
        decoder.receive("start 2 2");
        decoder.receive("re=0,0");
        decoder.receive("re=0,0");
        assert!(matches!(decoder.receive("end"), Some(ReadboardInbound::Frame(_))));

        // 2. Wrong row count (1 row instead of 2) -> Rejected
        decoder.receive("re=0,0");
        let res = decoder.receive("end");
        assert!(matches!(res, Some(ReadboardInbound::Rejected(_))));

        // Next valid frame still decodes (size remembered)
        decoder.receive("re=1,1");
        decoder.receive("re=2,2");
        assert!(matches!(decoder.receive("end"), Some(ReadboardInbound::Frame(_))));

        // 3. Wrong token count (3 tokens instead of 2) -> Rejected
        decoder.receive("re=0,0,0");
        decoder.receive("re=0,0");
        let res = decoder.receive("end");
        assert!(matches!(res, Some(ReadboardInbound::Rejected(_))));

        // Next valid frame still decodes
        decoder.receive("re=1,1");
        decoder.receive("re=2,2");
        assert!(matches!(decoder.receive("end"), Some(ReadboardInbound::Frame(_))));

        // 4. Token 5 -> Rejected
        decoder.receive("re=0,5");
        decoder.receive("re=0,0");
        let res = decoder.receive("end");
        assert!(matches!(res, Some(ReadboardInbound::Rejected(_))));

        // Next valid frame still decodes
        decoder.receive("re=1,1");
        decoder.receive("re=2,2");
        assert!(matches!(decoder.receive("end"), Some(ReadboardInbound::Frame(_))));

        // 5. Token 12 -> Rejected
        decoder.receive("re=12,0");
        decoder.receive("re=0,0");
        let res = decoder.receive("end");
        assert!(matches!(res, Some(ReadboardInbound::Rejected(_))));

        // Next valid frame still decodes
        decoder.receive("re=1,1");
        decoder.receive("re=2,2");
        let Some(ReadboardInbound::Frame(frame)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame.codes, vec![1, 1, 2, 2]);
    }

    #[test]
    fn test_frame_interrupted_by_control_lines() {
        let mut decoder = FrameDecoder::new();
        decoder.receive("start 2 2");

        // Interrupted by clear
        decoder.receive("re=1,1");
        assert_eq!(
            decoder.receive("clear"),
            Some(ReadboardInbound::Control(ReadboardControl::Clear))
        );
        assert!(matches!(
            decoder.receive("end"),
            Some(ReadboardInbound::Rejected(_))
        ));

        // Interrupted by start
        decoder.receive("re=1,1");
        assert_eq!(
            decoder.receive("start 2 2"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((2, 2))
            }))
        );
        assert!(matches!(
            decoder.receive("end"),
            Some(ReadboardInbound::Rejected(_))
        ));

        // Interrupted by stopsync
        decoder.receive("re=1,1");
        assert_eq!(
            decoder.receive("stopsync"),
            Some(ReadboardInbound::Control(ReadboardControl::StopSync))
        );
        assert!(matches!(
            decoder.receive("end"),
            Some(ReadboardInbound::Rejected(_))
        ));
    }

    #[test]
    fn test_endsync_emits_control_and_resets() {
        let mut decoder = FrameDecoder::new();
        decoder.receive("start 2 2");
        decoder.receive("re=1,1");
        assert_eq!(
            decoder.receive("endsync"),
            Some(ReadboardInbound::Control(ReadboardControl::EndSync))
        );
        // Rows cleared, so end rejects
        assert!(matches!(
            decoder.receive("end"),
            Some(ReadboardInbound::Rejected(_))
        ));
    }

    #[test]
    fn test_context_does_not_leak_across_end() {
        let mut decoder = FrameDecoder::new();
        decoder.receive("start 2 2");
        decoder.receive("syncPlatform fox");
        decoder.receive("roomToken room-1");
        decoder.receive("liveTitleMove 42");
        decoder.receive("forceRebuild");
        decoder.receive("foxMoveNumber 42");
        decoder.receive("lastMoveSource redBlueMarker");
        decoder.receive("re=0,0");
        decoder.receive("re=0,0");
        let Some(ReadboardInbound::Frame(frame1)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame1.context.platform, ReadboardPlatform::Fox);
        assert_eq!(frame1.context.room_token.as_deref(), Some("room-1"));
        assert_eq!(frame1.context.live_title_move, Some(42));
        assert!(frame1.context.force_rebuild);
        assert_eq!(frame1.context.fox_move_number, Some(42));
        assert_eq!(
            frame1.context.last_move_source,
            ReadboardLastMoveSource::RedBlueMarker
        );

        // Frame 2 with no context lines
        decoder.receive("re=0,0");
        decoder.receive("re=0,0");
        let Some(ReadboardInbound::Frame(frame2)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame2.context.platform, ReadboardPlatform::Generic);
        assert_eq!(frame2.context.window_kind, ReadboardWindowKind::Unknown);
        assert_eq!(frame2.context.room_token, None);
        assert_eq!(frame2.context.live_title_move, None);
        assert!(!frame2.context.force_rebuild);
        assert_eq!(frame2.context.fox_move_number, None);
        assert_eq!(
            frame2.context.last_move_source,
            ReadboardLastMoveSource::LegacyUnknown
        );
    }

    #[test]
    fn test_trailing_cr_and_whitespace() {
        let mut decoder = FrameDecoder::new();
        assert_eq!(
            decoder.receive("start 2 2\r"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((2, 2))
            }))
        );
        assert_eq!(decoder.receive("syncPlatform fox\r"), None);
        assert_eq!(decoder.receive("re=1,2\r"), None);
        assert_eq!(decoder.receive("re=3,4\r"), None);
        let Some(ReadboardInbound::Frame(frame)) = decoder.receive("end\r") else {
            panic!("expected Frame");
        };
        assert_eq!(frame.codes, vec![1, 2, 3, 4]);
        assert_eq!(frame.context.platform, ReadboardPlatform::Fox);
    }

    #[test]
    fn test_start_size_validation_and_persistence() {
        let mut decoder = FrameDecoder::new();

        // Valid size with hwnd
        assert_eq!(
            decoder.receive("start 19 19 0x12345"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((19, 19))
            }))
        );

        // Invalid start lines: keep previous size (19, 19)
        assert_eq!(
            decoder.receive("start"),
            Some(ReadboardInbound::Control(ReadboardControl::Start { size: None }))
        );
        assert_eq!(
            decoder.receive("start 1 19"),
            Some(ReadboardInbound::Control(ReadboardControl::Start { size: None }))
        );
        assert_eq!(
            decoder.receive("start 26 19"),
            Some(ReadboardInbound::Control(ReadboardControl::Start { size: None }))
        );
        assert_eq!(
            decoder.receive("start abc def"),
            Some(ReadboardInbound::Control(ReadboardControl::Start { size: None }))
        );

        // Size (19, 19) is still active:
        for _ in 0..19 {
            decoder.receive(&format!("re={}", vec!["0"; 19].join(",")));
        }
        assert!(matches!(decoder.receive("end"), Some(ReadboardInbound::Frame(_))));

        // Boundary valid sizes 2..=25
        assert_eq!(
            decoder.receive("start 2 25"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((2, 25))
            }))
        );
        assert_eq!(
            decoder.receive("start 25 2"),
            Some(ReadboardInbound::Control(ReadboardControl::Start {
                size: Some((25, 2))
            }))
        );
    }

    #[test]
    fn test_clearboard_and_sync() {
        let mut decoder = FrameDecoder::new();
        assert_eq!(
            decoder.receive("clearBoard"),
            Some(ReadboardInbound::Control(ReadboardControl::ClearBoard))
        );
        assert_eq!(
            decoder.receive("sync"),
            Some(ReadboardInbound::Control(ReadboardControl::Sync))
        );

        // clearBoard does not reset pending rows
        decoder.receive("start 2 2");
        decoder.receive("re=1,1");
        assert_eq!(
            decoder.receive("clearBoard"),
            Some(ReadboardInbound::Control(ReadboardControl::ClearBoard))
        );
        decoder.receive("re=2,2");
        let Some(ReadboardInbound::Frame(frame)) = decoder.receive("end") else {
            panic!("expected Frame");
        };
        assert_eq!(frame.codes, vec![1, 1, 2, 2]);
    }

    #[test]
    fn test_ignored_lines() {
        let mut decoder = FrameDecoder::new();
        assert_eq!(decoder.receive("version: 220430"), None);
        assert_eq!(decoder.receive("ready"), None);
        assert_eq!(decoder.receive("playponder on"), None);
        assert_eq!(decoder.receive("error place failed"), None);
        assert_eq!(decoder.receive(""), None);
        assert_eq!(decoder.receive("   "), None);
        assert_eq!(decoder.receive("startfoo 19 19"), None);
    }
}
