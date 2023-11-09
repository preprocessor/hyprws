use anyhow::{anyhow, Result};
use hyprland::data::*;
use hyprland::dispatch::*;
use hyprland::prelude::*;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") => help(),
        Some("--swap") | Some("-s") => swap_windows(args.get(2), args.get(3))?,
        Some("--dump") | Some("-d") => dump_windows(args.get(2), args.get(3))?,
        Some("--kill") | Some("-k") => kill_workspace(args.get(2))?,
        Some(other) => {
            error_message(&format!("Invalid argument: {}", other));
            help(); // Default to help message
        }
        None => {
            error_message("No arguments provided");
            help(); // Default to help message
        }
    }

    Ok(())
}

fn help() {
    // Display help message
    println!("Usage:");
    println!("  hyprws [OPTION] [WORKSPACE(S)]");
    println!("Options:");
    println!("  [ --help, -h ] WS1 WS2  Display this help message");
    println!("  [ --swap, -s ] WS1 WS2  Swap windows on Workspace1 with Workspace2");
    println!("  [ --dump, -d ] WS1 WS2  Move all windows from one workspace to another");
    println!("  [ --kill, -k ] WS       Close all windows on one workspace");
}

fn error_message(message: &str) {
    eprintln!("Error: {}", message);
}

fn valid_workspaces(a: Option<&String>, b: Option<&String>) -> Result<(i32, i32)> {
    match (
        a.and_then(|s| s.parse::<i32>().ok()),
        b.and_then(|s| s.parse::<i32>().ok()),
    ) {
        (Some(a), Some(b)) => Ok((a, b)), //  Both a and b can be parsed as numbers
        _ => Err(anyhow!("Invalid workspace ID")), // At least one of them cannot be parsed as a number
    }
}

/// Filter clients by workspace ID and pinned status
fn filter_clients(target_workspace_id: i32) -> Result<Vec<Client>> {
    Ok(Clients::get()?
        .filter(|client| client.workspace.id == target_workspace_id && !client.pinned)
        .collect())
}

fn kill_workspace(target: Option<&String>) -> Result<()> {
    let (target, _) = valid_workspaces(target, target)?;

    let clients = filter_clients(target)?;

    for client in clients {
        Dispatch::call(DispatchType::CloseWindow(WindowIdentifier::Address(
            client.address,
        )))?;
    }

    Ok(())
}

fn dump_windows(start: Option<&String>, end: Option<&String>) -> Result<()> {
    let (start, end) = valid_workspaces(start, end)?;
    let clients = filter_clients(start)?;

    for client in clients {
        Dispatch::call(DispatchType::MoveToWorkspaceSilent(
            WorkspaceIdentifierWithSpecial::Id(end),
            Some(WindowIdentifier::Address(client.address)),
        ))?;
    }

    Ok(())
}

fn swap_windows(start: Option<&String>, end: Option<&String>) -> Result<()> {
    let (start, end) = valid_workspaces(start, end)?;
    let start_clients = filter_clients(start)?;
    let end_clients = filter_clients(end)?;

    for client in start_clients {
        Dispatch::call(DispatchType::MoveToWorkspaceSilent(
            WorkspaceIdentifierWithSpecial::Id(end),
            Some(WindowIdentifier::Address(client.address)),
        ))?;
    }

    for client in end_clients {
        Dispatch::call(DispatchType::MoveToWorkspaceSilent(
            WorkspaceIdentifierWithSpecial::Id(start),
            Some(WindowIdentifier::Address(client.address)),
        ))?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use anyhow::Ok;

    use super::*;

    #[test]
    fn dump() -> Result<()> {
        dump_windows(Some(&"6".to_string()), Some(&"9".to_string()))?;
        Ok(())
    }

    #[test]
    fn swap() -> Result<()> {
        swap_windows(Some(&"6".to_string()), Some(&"9".to_string()))?;
        Ok(())
    }
}
