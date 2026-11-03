use super::{CliResult, RepoArg, path_str};
use crate::mcp::OutcomeInput;
use crate::store::NewOutcome;
use quick_xml::{Reader, events::Event};
use sha2::{Digest, Sha256};
use std::path::Path;

/// More failing cases than this in one report are counted, not stored: a broken build
/// should not turn one record into hundreds of outcomes.
const MAX_FAILURES: usize = 20;
const MAX_REPORT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, PartialEq)]
pub(crate) struct Case {
    pub id: String,
    pub failed: bool,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Parsed {
    pub cases: Vec<Case>,
    pub skipped: usize,
    /// Passed only after a retry (nextest `flakyFailure`); not evidence either way.
    pub flaky: usize,
}

/// Read a JUnit report (nextest, pytest, Maven, Jest and most other runners write this
/// shape). Only test names and pass/fail are kept; failure messages and captured output
/// are dropped on purpose, so nothing from the report except those names is stored.
pub(crate) fn parse(xml: &str) -> Result<Parsed, String> {
    let mut reader = Reader::from_str(xml);
    let mut out = Parsed::default();
    // (id, failed, skipped, flaky) of the open <testcase>
    let mut open: Option<(String, bool, bool, bool)> = None;
    let mut seen_root = false;
    loop {
        match reader.read_event().map_err(|e| format!("not a valid XML report: {e}"))? {
            ev @ (Event::Start(_) | Event::Empty(_)) => {
                let is_empty = matches!(ev, Event::Empty(_));
                let (Event::Start(e) | Event::Empty(e)) = ev else { unreachable!() };
                let name = e.local_name();
                match name.as_ref() {
                    b"testsuites" | b"testsuite" => seen_root = true,
                    b"testcase" => {
                        let mut class = String::new();
                        let mut test = String::new();
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().map_err(|e| e.to_string())?.into_owned();
                            match a.key.local_name().as_ref() {
                                b"classname" => class = v,
                                b"name" => test = v,
                                _ => {}
                            }
                        }
                        let id = if class.is_empty() { test } else { format!("{class}::{test}") };
                        open = Some((id, false, false, false));
                    }
                    b"failure" | b"error" => {
                        if let Some(c) = open.as_mut() {
                            c.1 = true;
                        }
                    }
                    b"skipped" => {
                        if let Some(c) = open.as_mut() {
                            c.2 = true;
                        }
                    }
                    b"flakyFailure" | b"flakyError" => {
                        if let Some(c) = open.as_mut() {
                            c.3 = true;
                        }
                    }
                    _ => {}
                }
                // A self-closing <testcase .../> has no End event.
                if is_empty
                    && name.as_ref() == b"testcase"
                    && let Some((id, failed, skipped, flaky)) = open.take()
                {
                    close(&mut out, id, failed, skipped, flaky);
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"testcase" => {
                if let Some((id, failed, skipped, flaky)) = open.take() {
                    close(&mut out, id, failed, skipped, flaky);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !seen_root {
        return Err("no <testsuite> element: this does not look like a JUnit report".into());
    }
    Ok(out)
}

fn close(out: &mut Parsed, id: String, failed: bool, skipped: bool, flaky: bool) {
    if skipped && !failed {
        out.skipped += 1;
    } else if flaky && !failed {
        out.flaky += 1;
    } else {
        out.cases.push(Case { id, failed });
    }
}

/// Attach the failing cases of a JUnit report (and, with `--test`, matching passing
/// ones) to a record. The report is evidence only as far as the user pointed Vexen
/// at it: it is a declared result, not a run Vexen watched.
pub fn run(repo: &RepoArg, id: String, report: &Path, filter: Option<String>) -> CliResult {
    let root = repo.root()?;
    let meta = std::fs::metadata(report).map_err(|e| format!("cannot read {}: {e}", report.display()))?;
    if meta.len() > MAX_REPORT_BYTES {
        return Err(format!("{} is larger than {} MB", report.display(), MAX_REPORT_BYTES / 1024 / 1024).into());
    }
    let bytes = std::fs::read(report).map_err(|e| format!("cannot read {}: {e}", report.display()))?;
    let xml = String::from_utf8(bytes.clone()).map_err(|_| "the report is not UTF-8")?;
    let parsed = parse(&xml)?;
    let report_sha: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();

    let wanted = |c: &Case| match &filter {
        Some(f) => c.id.contains(f.as_str()),
        None => c.failed,
    };
    let mut chosen: Vec<&Case> = parsed.cases.iter().filter(|c| wanted(c)).collect();
    let dropped = chosen.len().saturating_sub(MAX_FAILURES);
    chosen.truncate(MAX_FAILURES);
    if chosen.is_empty() {
        println!(
            "vexen: nothing recorded ({} tests, {} skipped, {} flaky). Without --test only failing tests are attached; \
             pass --test <name-part> to attach a passing one.",
            parsed.cases.len(),
            parsed.skipped,
            parsed.flaky
        );
        return Ok(());
    }
    let (mut failing, mut passing) = (0, 0);
    for case in chosen {
        let exit_code = i32::from(case.failed);
        let mut h = Sha256::new();
        h.update(case.id.as_bytes());
        h.update(exit_code.to_le_bytes());
        h.update(report_sha.as_bytes());
        let key: String = h.finalize().iter().take(16).map(|b| format!("{b:02x}")).collect();
        let test_id: String = case.id.chars().filter(|c| !c.is_control()).take(500).collect();
        crate::mcp::outcome(OutcomeInput {
            repo_root: path_str(&root),
            event_id: id.clone(),
            outcome: Some(NewOutcome { idempotency_key: key, test_id, exit_code, output_sha256: Some(report_sha.clone()) }),
            supersede: None,
        })?;
        if case.failed { failing += 1 } else { passing += 1 }
    }
    println!(
        "vexen: attached {failing} failing and {passing} passing test result(s) from {}; \
         {dropped} more failing over the limit of {MAX_FAILURES}, {} skipped, {} flaky left out. \
         This is a result you supplied, not a run Vexen observed.",
        report.display(),
        parsed.skipped,
        parsed.flaky
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = r#"<?xml version="1.0"?>
<testsuites><testsuite name="vexen" tests="5">
  <testcase classname="vexen::retry" name="no_double_charge"><failure message="boom">secret-looking output</failure></testcase>
  <testcase classname="vexen::retry" name="ok_case"/>
  <testcase classname="vexen::retry" name="slow"><skipped/></testcase>
  <testcase classname="vexen::retry" name="flaky"><flakyFailure message="x"/></testcase>
  <testcase name="plain_error"><error type="E">oops</error></testcase>
</testsuite></testsuites>"#;

    #[test]
    fn reads_failures_passes_skips_and_flaky_without_keeping_messages() {
        let p = parse(REPORT).unwrap();
        assert_eq!(
            p.cases,
            vec![
                Case { id: "vexen::retry::no_double_charge".into(), failed: true },
                Case { id: "vexen::retry::ok_case".into(), failed: false },
                Case { id: "plain_error".into(), failed: true },
            ]
        );
        assert_eq!((p.skipped, p.flaky), (1, 1));
    }

    #[test]
    fn a_file_that_is_not_a_junit_report_is_refused() {
        assert!(parse("<html><body/></html>").is_err());
        assert!(parse("not xml at all <<<").is_err());
    }
}
