//! The admitted kata-raw-nn 0 field, not an ordinary analysis frame.
pub(crate) fn admitted(resource: &app_model::QualifiedLocalResourceDto) -> bool {
    crate::thread_policy::direct_local(resource)
        && resource.resources.iter().any(|item| {
            item.component == "model"
                && item.sha256 == "7c8a84ed9ee737e9c7e741a08bf242d63db37b648e7f64942f3a8b1b5101e7c2"
        })
        && resource.resources.iter().any(|item| {
            item.component == "executable"
                && matches!(
                    item.sha256.as_str(),
                    "20c4ede9f39455ab3c019ada7a22a6e4d7696361161feed5c56f2c42d1507bd0"
                        | "38f6addb15f4b16d0417f3aaf55bdbcad6c624f0c665e80a2d317a11a675a6c0"
                )
        })
}

pub(crate) fn ownership(body: &str, width: u8, height: u8) -> Result<Vec<f32>, String> {
    if width == 0 || height == 0 {
        return Err("Raw-net ownership geometry is empty".into());
    }
    let mut lines = body.lines().peekable();
    let mut result: Option<Vec<f64>> = None;
    while let Some(line) = lines.next() {
        let mut fields = line.split_whitespace();
        if fields.next() != Some("whiteOwnership") {
            continue;
        }
        if fields.next().is_some() {
            return Err("Malformed raw-net whiteOwnership header".into());
        }
        let mut values = Vec::with_capacity(usize::from(width) * usize::from(height));
        for _ in 0..height {
            let row = lines.next().ok_or("Incomplete raw-net ownership geometry")?;
            let start = values.len();
            for token in row.split_whitespace() {
                let value: f64 = token.parse().map_err(|_| "Malformed raw-net ownership value")?;
                if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                    return Err("Raw-net ownership value is not finite or in [-1,1]".into());
                }
                if values.len() - start == usize::from(width) {
                    return Err("Raw-net ownership row has the wrong width".into());
                }
                values.push(-value);
            }
            if values.len() - start != usize::from(width) {
                return Err("Raw-net ownership row has the wrong width".into());
            }
        }
        if lines
            .peek()
            .and_then(|line| line.split_whitespace().next())
            .is_some_and(|word| word.parse::<f32>().is_ok())
        {
            return Err("Raw-net ownership has too many rows".into());
        }
        if result.as_ref().is_some_and(|old| old != &values) {
            return Err("Conflicting duplicate raw-net ownership fields".into());
        }
        result = Some(values);
    }
    result
        .map(|values| values.into_iter().map(|value| value as f32).collect())
        .ok_or_else(|| "Raw-net whiteOwnership is unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::ownership;

    #[test]
    fn rectangular_white_values_become_black_positive_without_turn_flip() {
        let body =
            "symmetry 0\npolicy\nNAN NAN\nwhiteOwnership\n-0.25 0 1\n0.5 -1 -0.75\nfutureField anything";
        assert_eq!(ownership(body, 3, 2).unwrap(), [0.25, 0.0, -1.0, -0.5, 1.0, 0.75]);
        assert!(ownership(body, 2, 3).is_err());
    }

    #[test]
    fn missing_malformed_nonfinite_range_and_geometry_are_unavailable() {
        for body in [
            "policy\nNAN",
            "whiteOwnership",
            "whiteOwnership 0",
            "whiteOwnership\n0",
            "whiteOwnership\n0 0 0",
            "whiteOwnership\n0 0\n0 0",
            "whiteOwnership\nNaN 0",
            "whiteOwnership\ninf 0",
            "whiteOwnership\n1.00000001 0",
            "whiteOwnership\n-1.01 0",
            "whiteOwnership\ninvalid 0",
        ] {
            assert!(ownership(body, 2, 1).is_err(), "{body}");
        }
        assert!(ownership("whiteOwnership\n0", 0, 1).is_err());
    }

    #[test]
    fn duplicate_conflicts_reject_but_unknown_fields_do_not() {
        assert_eq!(
            ownership("whiteOwnership\n0 1\nnewField 5\nwhiteOwnership\n0 1", 2, 1).unwrap(),
            [0.0, -1.0]
        );
        assert!(ownership("whiteOwnership\n0 1\nwhiteOwnership\n0 -1", 2, 1).is_err());
        assert!(ownership("whiteOwnership\n0 0.5\nwhiteOwnership\n0 0.500000001", 2, 1).is_err());
    }
}
