use std::{collections::HashMap, process::ExitCode};

use ai::{
    funcs::{FunctionArguments, FunctionBuilder},
    history::FileHistory,
    Chat, OpenAiPlatform,
};
use anyhow::{anyhow, Context};
use dialoguer::Confirm;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;
use tracing::debug;

const USEFUL_ENV: &[&str] = &["LANG", "PWD", "HOME", "PATH", "TERM"];

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = run().await {
        eprintln!("{:?}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct Script<'a> {
    kind: &'a str,
    command: &'a str,
    content: &'a str,
}

async fn execute_script(script: Script<'_>) -> anyhow::Result<String> {
    if !Confirm::new()
        .with_prompt(format!(
            "I need to execute the following {} script:\n\n{}\n\nDo you want to proceed?",
            script.kind, script.content
        ))
        .interact()
        .context("Failed to ask user if script execution is allowed.")?
    {
        return Err(anyhow!("Aborted by user."));
    }

    let output = Command::new(script.command)
        .arg("-c")
        .arg(script.content)
        .output()
        .await
        .context("Script execution failed.")?;

    let output = ExecuteShellScriptFunctionOutput {
        status: output.status.code(),
        stdout: String::from_utf8(output.stdout)
            .context("Script executed sucessfully, but stdout is not utf-8 encoded.")?,
        stderr: String::from_utf8(output.stderr)
            .context("Script executed sucessfully, but stderr is not utf-8 encoded.")?,
    };
    debug!(output =? output, "Script executed.");

    anyhow::Ok(
        serde_json::to_string(&output)
            .context("Script executed sucessfully, but I've failed to serialize its output.")?,
    )
}

#[derive(Deserialize)]
struct ExecuteShellScriptFunctionArgs {
    script: String,
}

impl FunctionArguments for ExecuteShellScriptFunctionArgs {
    fn json_schema() -> Option<serde_json::Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "script": {
                    "type": "string",
                    "description": "Shell script to execute.",
                }
            }
        }))
    }
}

async fn execute_shell_script(args: ExecuteShellScriptFunctionArgs) -> anyhow::Result<String> {
    #[cfg(target_os = "windows")]
    let script = Script {
        kind: "powershell",
        command: "powershell",
        content: &args.script,
    };

    #[cfg(target_os = "linux")]
    let script = Script {
        kind: "shell",
        command: "sh",
        content: &args.script,
    };

    execute_script(script).await
}

#[derive(Deserialize)]
struct ExecutePythonScriptFunctionArgs {
    script: String,
}

impl FunctionArguments for ExecutePythonScriptFunctionArgs {
    fn json_schema() -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "script": {
                    "type": "string",
                    "description": "Python script to execute.",
                },
            },
        }))
    }
}

async fn execute_python_script(args: ExecutePythonScriptFunctionArgs) -> anyhow::Result<String> {
    execute_script(Script {
        kind: "python",
        command: "python",
        content: &args.script,
    })
    .await
}

fn useful_env() -> HashMap<String, String> {
    std::env::vars()
        .filter(|(k, _)| USEFUL_ENV.contains(&k.as_str()))
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

#[cfg(target_os = "windows")]
fn os_info() -> Result<Value, sys_info::Error> {
    Ok(json!({
        "name": "Windows",
        "release": sys_info::os_release()?,
        "env": useful_env(),
    }))
}

#[cfg(target_os = "linux")]
fn os_info() -> Result<Value, sys_info::Error> {
    let info = sys_info::linux_os_release()?;

    Ok(json!({
        "name": info.name.unwrap_or_else(|| "linux".into()),
        "version": info.version,
        "variant": info.variant,
        "env": useful_env(),
    }))
}

#[derive(Debug, Serialize)]
struct ExecuteShellScriptFunctionOutput {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let history_path = ProjectDirs::from("dev", "magmast", "ai")
        .context("Failed to get project dirs.")?
        .state_dir()
        .context("Failed to get state dir.")?
        .join("history.json");

    let mut chat = Chat::<OpenAiPlatform, _>::from(FileHistory::new(history_path));

    chat.system_message(Some(
        json!({
            "message": include_str!("../../../assets/system_message.txt"),
            "os": os_info().context("Failed to get os info.")?,
        })
        .to_string(),
    ))
    .function(
        Into::<FunctionBuilder<_, _, _>>::into(execute_shell_script).description(Some(
            include_str!("../../../assets/execute_shell_script_description.txt").into(),
        )),
    )
    .function(
        Into::<FunctionBuilder<_, _, _>>::into(execute_python_script).description(Some(
            include_str!("../../../assets/execute_python_script_description.txt").into(),
        )),
    );

    let response = chat
        .send(
            std::env::args()
                .skip(1)
                .reduce(|acc, arg| format!("{} {}", acc, arg))
                .context("Sorry, but I cannot help, if your message is empty.")?,
        )
        .await?;

    println!("{response}");

    Ok(())
}
