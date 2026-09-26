use project_engine::artifact_integrity::sha256_hex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "literary-reveal-authority-v1@persian-literary-translation-engine";
const PROTOCOL: &str = "PLTE-REVEAL-AUTHORITY-V1";

type Result<T> = std::result::Result<T, String>;

fn usage() -> String {
    "Usage:\n  literary-engine blind-review sign-reveal-authority <bundle> <reveal-key> <signature> --project <id> --review <id> --authority <principal> --key <ssh-key>\n  literary-engine blind-review verify-reveal-authority <bundle> <reveal-key> <signature> --project <id> --review <id> --authority <principal> --allowed-signers <file> [--revocations <file>]".into()
}

pub(crate) fn run(args: &[String]) -> Result<()> {
    if args.len() < 4 {
        return Err(usage());
    }
    let command = args[0].as_str();
    let bundle = &args[1];
    let reveal = &args[2];
    let signature = &args[3];
    let mut options = args[4..].chunks_exact(2);
    for pair in &mut options {
        if !matches!(
            pair[0].as_str(),
            "--project"
                | "--review"
                | "--authority"
                | "--key"
                | "--allowed-signers"
                | "--revocations"
        ) || pair[1].starts_with("--")
            || pair[1].is_empty()
        {
            return Err("invalid reveal-authority option or value".into());
        }
    }
    if !options.remainder().is_empty() {
        return Err("reveal-authority option requires a value".into());
    }
    require_regular(bundle, "bundle")?;
    require_regular(reveal, "reveal key")?;
    ensure_distinct(bundle, reveal)?;

    let project = flag(args, "--project", true)?.unwrap();
    let review = flag(args, "--review", true)?.unwrap();
    let authority = flag(args, "--authority", true)?.unwrap();
    validate_atom(project, "project")?;
    validate_atom(review, "review")?;
    validate_atom(authority, "authority")?;

    let bundle_bytes = fs::read(bundle).map_err(|e| format!("failed to read bundle: {e}"))?;
    let reveal_bytes = fs::read(reveal).map_err(|e| format!("failed to read reveal key: {e}"))?;
    let statement = authority_statement(project, review, authority, &bundle_bytes, &reveal_bytes);

    match command {
        "sign" => {
            let key = flag(args, "--key", true)?.unwrap();
            reject_flag(args, "--allowed-signers")?;
            reject_flag(args, "--revocations")?;
            require_regular(key, "signing key")?;
            ensure_distinct(signature, bundle)?;
            ensure_distinct(signature, reveal)?;
            ensure_distinct(signature, key)?;
            if Path::new(signature).symlink_metadata().is_ok() {
                return Err(
                    "signature output already exists; refusing overwrite or symlink target".into(),
                );
            }
            let signed = crate::provider_review_cmd::ssh_sign_with_namespace(
                statement.as_bytes(),
                key,
                NAMESPACE,
            )?;
            write_new_atomic(Path::new(signature), &signed)?;
            println!("reveal-authority signature written: {signature}");
        }
        "verify" => {
            reject_flag(args, "--key")?;
            let allowed = flag(args, "--allowed-signers", true)?.unwrap();
            let revocations = flag(args, "--revocations", false)?;
            require_regular(signature, "signature")?;
            require_regular(allowed, "allowed-signers file")?;
            if let Some(path) = revocations {
                require_regular(path, "revocations file")?;
            }
            crate::provider_review_cmd::ssh_verify_with_namespace(
                statement.as_bytes(),
                signature,
                allowed,
                authority,
                revocations,
                NAMESPACE,
            )?;
            println!("reveal-authority signature verified: {authority}");
        }
        _ => return Err(usage()),
    }
    println!("signature namespace: {NAMESPACE}");
    println!("production admission: NOT GRANTED");
    Ok(())
}

fn authority_statement(
    project: &str,
    review: &str,
    authority: &str,
    bundle: &[u8],
    reveal: &[u8],
) -> String {
    format!(
        "{PROTOCOL}\nproject={project}\nreview={review}\nbundle_sha256={}\nreveal_sha256={}\nauthority={authority}\n",
        sha256_hex(bundle), sha256_hex(reveal)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_authority_statement_binds_exact_bytes_and_context() {
        let statement = authority_statement("p1", "r1", "editor", b"bundle", b"reveal");
        assert!(statement.starts_with("PLTE-REVEAL-AUTHORITY-V1\nproject=p1\nreview=r1\n"));
        assert!(statement.ends_with("authority=editor\n"));
        for changed in [
            authority_statement("p2", "r1", "editor", b"bundle", b"reveal"),
            authority_statement("p1", "r2", "editor", b"bundle", b"reveal"),
            authority_statement("p1", "r1", "other", b"bundle", b"reveal"),
            authority_statement("p1", "r1", "editor", b"bundle ", b"reveal"),
            authority_statement("p1", "r1", "editor", b"bundle", b"reveal "),
        ] {
            assert_ne!(statement, changed);
        }
    }

    #[test]
    fn reveal_authority_context_rejects_ambiguous_fields() {
        for invalid in ["", "p\nreview=other", " leading", "a/b", "é"] {
            assert!(validate_atom(invalid, "project").is_err());
        }
        assert!(validate_atom("project_1@example.test", "project").is_ok());
    }
}

fn flag<'a>(args: &'a [String], name: &str, required: bool) -> Result<Option<&'a str>> {
    let positions = args
        .iter()
        .enumerate()
        .filter(|(_, v)| v.as_str() == name)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if positions.len() > 1 {
        return Err(format!("duplicate flag: {name}"));
    }
    match positions.first() {
        Some(&i) => args
            .get(i + 1)
            .filter(|v| !v.starts_with("--") && !v.is_empty())
            .map(|v| Some(v.as_str()))
            .ok_or_else(|| format!("{name} requires a value")),
        None if required => Err(format!("missing required flag: {name}")),
        None => Ok(None),
    }
}

fn reject_flag(args: &[String], name: &str) -> Result<()> {
    if args.iter().any(|v| v == name) {
        Err(format!("{name} is not valid for this command"))
    } else {
        Ok(())
    }
}

fn validate_atom(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'@' | b':' | b'+' | b'-')
        })
    {
        return Err(format!(
            "{label} must be a restricted ASCII atom of 1..=128 bytes"
        ));
    }
    Ok(())
}

fn require_regular(path: &str, label: &str) -> Result<()> {
    let meta = fs::symlink_metadata(path)
        .map_err(|_| format!("{label} must be a regular non-symlink file"))?;
    if !meta.file_type().is_file() || meta.file_type().is_symlink() {
        return Err(format!("{label} must be a regular non-symlink file"));
    }
    Ok(())
}

fn absolute(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent =
        fs::canonicalize(parent).map_err(|e| format!("failed to resolve path parent: {e}"))?;
    let name = path
        .file_name()
        .ok_or_else(|| "path has no file name".to_string())?;
    Ok(parent.join(name))
}

fn ensure_distinct(left: &str, right: &str) -> Result<()> {
    if absolute(left)? == absolute(right)? {
        Err("evidence paths must be distinct".into())
    } else {
        Ok(())
    }
}

fn write_new_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            parent.display()
        ));
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock error: {e}"))?
        .as_nanos();
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("output path has no UTF-8 file name")?;
    let temp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("failed to create temporary signature: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("failed to write signature: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("failed to sync signature: {e}"))?;
        fs::hard_link(&temp, path)
            .map_err(|e| format!("failed to install signature without overwrite: {e}"))?;
        fs::remove_file(&temp).map_err(|e| format!("failed to remove temporary signature: {e}"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
