use std::{collections::HashMap, process::ExitCode};

use ai_chat::{
    funcs::{FunctionArguments, FunctionBuilder},
    history::FileHistory,
    Chat, OpenAiPlatform,
};
use ai_chat_derive::FunctionArguments;
use anyhow::{anyhow, Context};
use dialoguer::Confirm;
use directories::ProjectDirs;
use futures::{pin_mut, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;
use tracing::{debug, warn};
use tracing_subscriber::{prelude::*, Layer, Registry};

const USEFUL_ENV: &[&str] = &["LANG", "PWD", "HOME", "PATH", "TERM"];

#[tokio::main]
async fn main() -> ExitCode {
    let result =
        async { ProjectDirs::from("dev", "magmast", "ai").context("Failed to get project dirs.") }
            .and_then(|project_dirs| {
                init_logging(&project_dirs);
                run(project_dirs)
            })
            .await;

    if let Err(err) = result {
        eprintln!("{:?}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn run(project_dirs: ProjectDirs) -> anyhow::Result<()> {
    let history_path = project_dirs
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
    .function(execute_shell_script.description(include_str!(
        "../../../assets/execute_shell_script_description.txt"
    )))
    .function(execute_python_script.description(include_str!(
        "../../../assets/execute_python_script_description.txt"
    )));

    let responses = chat.send(
        std::env::args()
            .skip(1)
            .reduce(|acc, arg| format!("{} {}", acc, arg))
            .context("Sorry, but I cannot help, if your message is empty.")?,
    );

    pin_mut!(responses);

    while let Ok(response) = responses.next().await.context("Failed to send message.") {
        let response = response?;
        println!("{response}");
    }

    Ok(())
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

#[derive(Deserialize, FunctionArguments)]
struct ExecuteShellScriptFunctionArgs {
    #[arg(description = "Shell script to execute.")]
    script: String,
}

async fn execute_shell_script(args: ExecuteShellScriptFunctionArgs) -> Result<String, String> {
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

    execute_script(script).await.map_err(|err| err.to_string())
}

#[derive(Deserialize, FunctionArguments)]
struct ExecutePythonScriptFunctionArgs {
    #[arg(description = "Python script to execute.")]
    script: String,
}

async fn execute_python_script(args: ExecutePythonScriptFunctionArgs) -> Result<String, String> {
    execute_script(Script {
        kind: "python",
        command: "python",
        content: &args.script,
    })
    .await
    .map_err(|err| err.to_string())
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

fn init_logging(project_dirs: &ProjectDirs) {
    let stderr_layer =
        tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    let file_layer = project_dirs
        .state_dir()
        .context("Failed to get the log file path.")
        .map(|state_dir| state_dir.join("logs"))
        .map(|path| tracing_appender::rolling::daily(path, "ai"))
        .map(|appender| {
            tracing_subscriber::fmt::layer()
                .with_writer(appender)
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::TRACE)
        });

    let registry = Registry::default().with(stderr_layer);
    match file_layer {
        Ok(file_layer) => registry.with(file_layer).init(),
        Err(err) => {
            registry.init();
            warn!(%err, "Failed to open log file.");
        }
    }
}
