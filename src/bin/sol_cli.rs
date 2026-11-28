//! Sol Next Public Edition — Zero-Dependency Rust 2024 Non-Interactive CLI (`sol-cli`).

use sol_next_tools::{
    InstallOptions, JsonValue, RELEASE_TAG, cmd_install, cmd_launch, cmd_run, cmd_start_helper,
    cmd_status, cmd_verify_vdisplay,
};
use std::env;
use std::process::ExitCode;

fn print_help() {
    eprintln!(
        "sol-cli ({RELEASE_TAG}) — Sol Next Public Edition non-interactive Rust 2024 CLI\n\n\
         USAGE:\n\
           sol-cli [--serial <id>] [--json] <COMMAND> [OPTIONS]\n\n\
         COMMANDS:\n\
           status                               Inspect ADB devices, Sol Next install state, and display specs\n\
           install [--abi arm64|universal] [--mirror github|china|ghfast] [--apk <path>] [--grant-permissions]\n\
           launch [--tab chat|personal_twin|brain_cockpit|control] [--open-calls] [--open-virtual-phone]\n\
           start-helper                         Start uid=2000 1:1 Background VirtualDisplay shell helper\n\
           run --intent <text>                  Dispatch a natural-language task intent to Sol Next\n\
           verify-vdisplay                      Verify 1:1 VirtualDisplay resolution & DPI alignment"
    );
}

fn emit_output(payload: &JsonValue, as_json: bool) {
    if as_json {
        println!("{}", payload.to_pretty_string());
    } else if let JsonValue::Object(entries) = payload {
        for (k, v) in entries {
            match v {
                JsonValue::Str(s) => println!("{k}: {s}"),
                other => println!("{k}: {}", other.to_compact_string()),
            }
        }
    } else {
        println!("{}", payload.to_pretty_string());
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return if args.is_empty() { ExitCode::from(2) } else { ExitCode::SUCCESS };
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("sol-cli {RELEASE_TAG}");
        return ExitCode::SUCCESS;
    }

    let mut serial: Option<String> = None;
    let mut as_json = false;
    let mut command: Option<String> = None;
    let mut rest: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--json" {
            as_json = true;
            i += 1;
        } else if a == "--serial" {
            if i + 1 >= args.len() {
                eprintln!("--serial requires a value");
                return ExitCode::from(2);
            }
            serial = Some(args[i + 1].clone());
            i += 2;
        } else if command.is_none() && !a.starts_with('-') {
            command = Some(a.clone());
            i += 1;
        } else {
            rest.push(a.clone());
            i += 1;
        }
    }

    let Some(cmd) = command else {
        print_help();
        return ExitCode::from(2);
    };

    let result: Result<JsonValue, String> = match cmd.as_str() {
        "status" => Ok(cmd_status(serial.as_deref())),
        "install" => {
            let mut opts = InstallOptions {
                serial,
                abi: "arm64".to_string(),
                mirror: "github".to_string(),
                apk_path: None,
                verify_sha256: true,
                grant_permissions: false,
            };
            let mut j = 0;
            while j < rest.len() {
                match rest[j].as_str() {
                    "--abi" if j + 1 < rest.len() => {
                        opts.abi = rest[j + 1].clone();
                        j += 2;
                    }
                    "--mirror" if j + 1 < rest.len() => {
                        opts.mirror = rest[j + 1].clone();
                        j += 2;
                    }
                    "--apk" if j + 1 < rest.len() => {
                        opts.apk_path = Some(rest[j + 1].clone());
                        j += 2;
                    }
                    "--verify-sha256" => {
                        opts.verify_sha256 = true;
                        j += 1;
                    }
                    "--no-verify-sha256" => {
                        opts.verify_sha256 = false;
                        j += 1;
                    }
                    "--grant-permissions" => {
                        opts.grant_permissions = true;
                        j += 1;
                    }
                    other => {
                        return {
                            let err = JsonValue::obj(vec![
                                ("ok", JsonValue::Bool(false)),
                                ("error", JsonValue::Str(format!("unknown flag '{other}'"))),
                            ]);
                            emit_output(&err, as_json);
                            ExitCode::from(2)
                        };
                    }
                }
            }
            cmd_install(&opts)
        }
        "launch" => {
            let mut tab: Option<String> = None;
            let mut open_calls = false;
            let mut open_vp = false;
            let mut j = 0;
            while j < rest.len() {
                match rest[j].as_str() {
                    "--tab" if j + 1 < rest.len() => {
                        tab = Some(rest[j + 1].clone());
                        j += 2;
                    }
                    "--open-calls" => {
                        open_calls = true;
                        j += 1;
                    }
                    "--open-virtual-phone" => {
                        open_vp = true;
                        j += 1;
                    }
                    _ => j += 1,
                }
            }
            cmd_launch(serial.as_deref(), tab.as_deref(), open_calls, open_vp)
        }
        "start-helper" => cmd_start_helper(serial.as_deref()),
        "run" => {
            let mut intent: Option<String> = None;
            let mut j = 0;
            while j < rest.len() {
                if rest[j] == "--intent" && j + 1 < rest.len() {
                    intent = Some(rest[j + 1].clone());
                    j += 2;
                } else {
                    j += 1;
                }
            }
            match intent {
                Some(text) if !text.trim().is_empty() => cmd_run(serial.as_deref(), &text),
                _ => Err("--intent <text> is required".to_string()),
            }
        }
        "verify-vdisplay" => cmd_verify_vdisplay(serial.as_deref()),
        other => Err(format!("unknown command '{other}'")),
    };

    let payload = match result {
        Ok(val) => val,
        Err(err) => JsonValue::obj(vec![
            ("ok", JsonValue::Bool(false)),
            ("error", JsonValue::Str(err)),
            ("command", JsonValue::Str(cmd)),
        ]),
    };
    let ok = payload.get("ok").and_then(JsonValue::as_bool).unwrap_or(false);
    emit_output(&payload, as_json);
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
