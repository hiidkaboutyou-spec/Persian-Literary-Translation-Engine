use project_engine::artifact_integrity::sha256_hex;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const NAMESPACE: &str = "literary-reveal-authority-v1@persian-literary-translation-engine";
const PROTOCOL: &str = "PLTE-REVEAL-AUTHORITY-V1";

type Result<T> = std::result::Result<T, String>;

fn usage() -> String {
    "Usage:\n  reveal-authority sign <bundle> <reveal-key> <signature> --project <id> --review <id> --authority <principal> --key <ssh-key>\n  reveal-authority verify <bundle> <reveal-key> <signature> --project <id> --review <id> --authority <principal> --allowed-signers <file> [--revocations <file>]".into()
}

fn main() -> std::process::ExitCode {
    match run(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    if args.len() < 4 { return Err(usage()); }
    let command = args[0].as_str();
    let bundle = &args[1];
    let reveal = &args[2];
    let signature = &args[3];
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
    let statement = format!(
        "{PROTOCOL}\nproject={project}\nreview={review}\nbundle_sha256={}\nreveal_sha256={}\nauthority={authority}\n",
        sha256_hex(&bundle_bytes), sha256_hex(&reveal_bytes)
    );

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
                return Err("signature output already exists; refusing overwrite or symlink target".into());
            }
            let signed = ssh_sign(statement.as_bytes(), key)?;
            write_new_atomic(Path::new(signature), &signed)?;
            println!("reveal-authority signature written: {signature}");
        }
        "verify" => {
            reject_flag(args, "--key")?;
            let allowed = flag(args, "--allowed-signers", true)?.unwrap();
            let revocations = flag(args, "--revocations", false)?;
            require_regular(signature, "signature")?;
            require_regular(allowed, "allowed-signers file")?;
            if let Some(path) = revocations { require_regular(path, "revocations file")?; }
            ssh_verify(statement.as_bytes(), signature, allowed, authority, revocations)?;
            println!("reveal-authority signature verified: {authority}");
        }
        _ => return Err(usage()),
    }
    println!("signature namespace: {NAMESPACE}");
    println!("production admission: NOT GRANTED");
    Ok(())
}

fn flag<'a>(args: &'a [String], name: &str, required: bool) -> Result<Option<&'a str>> {
    let positions = args.iter().enumerate().filter(|(_, v)| v.as_str() == name).map(|(i, _)| i).collect::<Vec<_>>();
    if positions.len() > 1 { return Err(format!("duplicate flag: {name}")); }
    match positions.first() {
        Some(&i) => args.get(i + 1).filter(|v| !v.starts_with("--") && !v.is_empty()).map(|v| Some(v.as_str())).ok_or_else(|| format!("{name} requires a value")),
        None if required => Err(format!("missing required flag: {name}")),
        None => Ok(None),
    }
}

fn reject_flag(args: &[String], name: &str) -> Result<()> {
    if args.iter().any(|v| v == name) { Err(format!("{name} is not valid for this command")) } else { Ok(()) }
}

fn validate_atom(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 || !value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.'|b'_'|b'@'|b':'|b'+'|b'-')) {
        return Err(format!("{label} must be a restricted ASCII atom of 1..=128 bytes"));
    }
    Ok(())
}

fn require_regular(path: &str, label: &str) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|_| format!("{label} must be a regular non-symlink file"))?;
    if !meta.file_type().is_file() || meta.file_type().is_symlink() { return Err(format!("{label} must be a regular non-symlink file")); }
    Ok(())
}

fn absolute(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| format!("failed to resolve path parent: {e}"))?;
    let name = path.file_name().ok_or_else(|| "path has no file name".to_string())?;
    Ok(parent.join(name))
}

fn ensure_distinct(left: &str, right: &str) -> Result<()> {
    if absolute(left)? == absolute(right)? { Err("evidence paths must be distinct".into()) } else { Ok(()) }
}

fn ssh_sign(bytes: &[u8], key: &str) -> Result<Vec<u8>> {
    let mut child = Command::new("ssh-keygen").args(["-Y","sign","-f",key,"-n",NAMESPACE]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("failed to start ssh-keygen: {e}"))?;
    child.stdin.take().ok_or("failed to open signing stdin")?.write_all(bytes).map_err(|e| format!("failed to stream statement: {e}"))?;
    let output = child.wait_with_output().map_err(|e| format!("failed to wait for ssh-keygen: {e}"))?;
    if !output.status.success() { return Err(format!("reveal-authority signing failed: {}", bounded(&output.stderr))); }
    if !output.stdout.starts_with(b"-----BEGIN SSH SIGNATURE-----") { return Err("ssh-keygen returned an unexpected signature format".into()); }
    Ok(output.stdout)
}

fn ssh_verify(bytes: &[u8], signature: &str, allowed: &str, authority: &str, revocations: Option<&str>) -> Result<()> {
    let mut command = Command::new("ssh-keygen");
    command.args(["-Y","verify","-f",allowed,"-I",authority,"-n",NAMESPACE,"-s",signature]);
    if let Some(path) = revocations { command.args(["-r", path]); }
    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("failed to start ssh-keygen: {e}"))?;
    child.stdin.take().ok_or("failed to open verification stdin")?.write_all(bytes).map_err(|e| format!("failed to stream statement: {e}"))?;
    let output = child.wait_with_output().map_err(|e| format!("failed to wait for ssh-keygen: {e}"))?;
    if !output.status.success() { return Err(format!("reveal-authority verification failed: {}", bounded(&output.stderr))); }
    Ok(())
}

fn bounded(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr); let trimmed = text.trim();
    if trimmed.is_empty() { "ssh-keygen returned a non-zero exit status".into() } else { trimmed.chars().take(600).collect() }
}

fn write_new_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() { return Err(format!("output directory does not exist: {}", parent.display())); }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| format!("system clock error: {e}"))?.as_nanos();
    let name = path.file_name().and_then(|v| v.to_str()).ok_or("output path has no UTF-8 file name")?;
    let temp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp).map_err(|e| format!("failed to create temporary signature: {e}"))?;
        file.write_all(bytes).map_err(|e| format!("failed to write signature: {e}"))?;
        file.sync_all().map_err(|e| format!("failed to sync signature: {e}"))?;
        fs::hard_link(&temp, path).map_err(|e| format!("failed to install signature without overwrite: {e}"))?;
        fs::remove_file(&temp).map_err(|e| format!("failed to remove temporary signature: {e}"))?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temp); }
    result
}
