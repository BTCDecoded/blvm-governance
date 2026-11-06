//! Lock for the protocol and governance contract text.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn section(text: &str, start: &str, end: &str) -> String {
    let from = text
        .find(start)
        .unwrap_or_else(|| panic!("missing heading {start}"));
    let rest = &text[from + start.len()..];
    let until = rest.find(end).unwrap_or(rest.len());
    rest[..until].to_string()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn rust_sources(dir: &Path, hits: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()));
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, hits);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            hits.push(path);
        }
    }
}

#[test]
fn spec_sections_match_the_payload_and_governance_events() {
    let root = repo_root();
    let protocol = read(&root.join("../blvm-spec/PROTOCOL.md"));
    let payload = section(
        &protocol,
        "#### 10.1.1 Message Header Parsing",
        "### 10.2 Connection Management",
    );
    assert!(
        !payload.contains("32 \\times 10^6") && !payload.contains("32 × 10^6"),
        "section 10.1.1 still states a 32 million byte payload cap"
    );
    assert!(
        payload.contains("4,000,000"),
        "section 10.1.1 does not state the 4,000,000 byte payload cap"
    );

    let standard = section(
        &protocol,
        "### 9.2 Standard Transaction Rules",
        "### 9.3 Replace-By-Fee",
    );
    assert!(
        !standard.contains("83 bytes"),
        "section 9.2 still caps a standard OP_RETURN at 83 bytes"
    );
    assert!(
        standard.contains("MAX_STANDARD_TX_WEIGHT / 4") && standard.contains("100,000"),
        "section 9.2 does not state the standard OP_RETURN cap"
    );

    let mut sources = Vec::new();
    rust_sources(&root.join("src"), &mut sources);
    for path in sources {
        let text = read(&path);
        assert!(
            !text.contains("EconomicNodeRegistry") && !text.contains("VetoThresholdReached"),
            "{} names a registry or veto type",
            path.display()
        );
    }

    for name in ["API.md", "README.md"] {
        let text = read(&root.join(name));
        assert!(
            !text.contains("EconomicNodeRegistry") && !text.contains("VetoThresholdReached"),
            "{name} still names a registry or veto type"
        );
        for event in [
            "GovernanceProposalCreated",
            "GovernanceProposalVoted",
            "GovernanceProposalMerged",
            "NewBlock",
        ] {
            assert!(text.contains(event), "{name} does not name {event}");
        }
    }
}
