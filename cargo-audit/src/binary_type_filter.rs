//! Filters a `rustsec::Report` to remove advisories not applicable to the given binary type.
//! For example, Windows-only advisories should not be reported for ELF files.

use std::collections::BTreeSet;
use std::str::FromStr;

use once_cell::sync::OnceCell;
use rustsec::platforms::{OS, platform::PlatformReq};

pub fn filter_report_by_binary_type(binary_type: &binfarce::Format, report: &mut rustsec::Report) {
    let vulns = &mut report.vulnerabilities;
    assert_eq!(vulns.list.len(), vulns.count, "Internal logic error: Incorrect number of vulnerabilities in the report!");
    vulns.list.retain(|vuln| advisory_applicable_to_binary(binary_type, &vuln.affected) );
    vulns.count = vulns.list.len();
    vulns.found = vulns.list.len() != 0;
    // TODO: also filter warnings
}

fn advisory_applicable_to_binary(binary_type: &binfarce::Format, affected: &Option<rustsec::advisory::Affected>) -> bool {
    if let Some(affected) = affected {
        if affected.os.is_empty() {
            true // all platforms are affected if the "os" list is empty
        } else {
            at_least_one_os_runs_binary(binary_type, &affected.os)
        }
    } else {
        true // all platforms are affected if "affected" section is not specified in the TOML
    }
}

fn at_least_one_os_runs_binary(binary_type: &binfarce::Format, os_list: &[OS]) -> bool {
    use binfarce::Format::*;
    match binary_type { 
        PE => os_list.contains(&OS::Windows),
        Macho => os_list.iter().any(|os| apple_OSs().contains(os)), // O(n*log(n))
        Elf32 {byte_order: _} | Elf64 {byte_order: _} => {
            // For now we'll assume it's affected if the list contains something other than Windows or Apple OSs
            os_list.iter().any(|os| os != &OS::Windows && ! apple_OSs().contains(os))
            // TODO: this could be improved if we somehow keep track of which OS uses elf and which doesn't.
            // Sadly `rustc --print-cfg` doesn't expose this information.
            // Perhaps we can make `platforms` expose the `family` which can be `windows` or `unix` or `unknown`?
            // That way we can capture all the unix-likes as using ELF and discard everything else
        },
        Unknown => true, // might be possible for detection based on panic messages?
    }
}

#[allow(non_snake_case)]
fn apple_OSs() -> &'static BTreeSet<OS> {
    static INSTANCE: OnceCell<BTreeSet<OS>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let req = PlatformReq::from_str("*apple*").unwrap();
        req.matching_platforms().map(|p| p.target_os).collect()
    })
}