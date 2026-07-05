use std::{env, process::Command};

const MIN_KITTY_DND_VERSION: KittyVersion = KittyVersion {
    major: 0,
    minor: 47,
    patch: 0,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct KittyDndRuntime {
    enabled: bool,
    machine_id: Option<String>,
}

impl KittyDndRuntime {
    pub(super) const fn disabled() -> Self {
        Self {
            enabled: false,
            machine_id: None,
        }
    }

    pub(super) fn enabled(machine_id: Option<String>) -> Self {
        Self {
            enabled: true,
            machine_id,
        }
    }

    pub(in crate::runtime) const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(in crate::runtime) fn drag_machine_id(&self) -> Option<&str> {
        self.machine_id.as_deref()
    }
}

pub(in crate::runtime) fn detect_kitty_dnd_runtime() -> KittyDndRuntime {
    let env = RuntimeEnv::read();
    detect_from_env(&env, query_kitty_version, local_machine_id)
}

#[derive(Debug, Default)]
struct RuntimeEnv {
    term: Option<String>,
    term_program: Option<String>,
    kitty_window_id: bool,
    tmux: bool,
    zellij: bool,
    ssh_connection: bool,
    ssh_tty: bool,
}

impl RuntimeEnv {
    fn read() -> Self {
        Self {
            term: env::var("TERM").ok(),
            term_program: env::var("TERM_PROGRAM").ok(),
            kitty_window_id: env::var_os("KITTY_WINDOW_ID").is_some(),
            tmux: env::var_os("TMUX").is_some(),
            zellij: env::var_os("ZELLIJ").is_some(),
            ssh_connection: env::var_os("SSH_CONNECTION").is_some(),
            ssh_tty: env::var_os("SSH_TTY").is_some(),
        }
    }
}

fn detect_from_env(
    env: &RuntimeEnv,
    query_version: impl FnOnce() -> Option<KittyVersion>,
    _read_machine_id: impl FnOnce() -> Option<String>,
) -> KittyDndRuntime {
    if !is_kitty_env(env) || env.tmux || env.zellij || env.ssh_connection || env.ssh_tty {
        return KittyDndRuntime::disabled();
    }

    match query_version() {
        Some(version) if version >= MIN_KITTY_DND_VERSION => KittyDndRuntime::enabled(None),
        _ => KittyDndRuntime::disabled(),
    }
}

fn is_kitty_env(env: &RuntimeEnv) -> bool {
    env.term
        .as_deref()
        .is_some_and(|term| term.to_ascii_lowercase().contains("xterm-kitty"))
        || env
            .term_program
            .as_deref()
            .is_some_and(|program| program.eq_ignore_ascii_case("kitty"))
        || env.kitty_window_id
}

fn query_kitty_version() -> Option<KittyVersion> {
    let output = Command::new("kitty").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_kitty_version(&String::from_utf8_lossy(&output.stdout))
        .or_else(|| parse_kitty_version(&String::from_utf8_lossy(&output.stderr)))
}

fn local_machine_id() -> Option<String> {
    None
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct KittyVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

fn parse_kitty_version(output: &str) -> Option<KittyVersion> {
    let version = output
        .split_whitespace()
        .find(|part| part.chars().next().is_some_and(|c| c.is_ascii_digit()))?;
    let mut parts = version.split('.');
    Some(KittyVersion {
        major: parts.next()?.parse().ok()?,
        minor: parts.next()?.parse().ok()?,
        patch: parts
            .next()
            .and_then(|patch| {
                patch
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kitty_env() -> RuntimeEnv {
        RuntimeEnv {
            term: Some("xterm-kitty".to_string()),
            ..RuntimeEnv::default()
        }
    }

    #[test]
    fn gates_to_kitty_047_or_newer() {
        assert!(
            detect_from_env(
                &kitty_env(),
                || parse_kitty_version("kitty 0.47.0"),
                || Some("local".to_string())
            )
            .is_enabled()
        );
        assert!(
            detect_from_env(
                &kitty_env(),
                || parse_kitty_version("kitty 0.47.4"),
                || Some("local".to_string())
            )
            .is_enabled()
        );
        assert!(
            !detect_from_env(
                &kitty_env(),
                || parse_kitty_version("kitty 0.46.2"),
                || Some("local".to_string())
            )
            .is_enabled()
        );
    }

    #[test]
    fn uses_empty_machine_id_for_local_drag_out() {
        let runtime = detect_from_env(
            &kitty_env(),
            || parse_kitty_version("kitty 0.47.4"),
            || Some("host;bad\x1b\\".to_string()),
        );
        assert_eq!(runtime.drag_machine_id(), None);
    }

    #[test]
    fn disables_without_queryable_version() {
        assert!(!detect_from_env(&kitty_env(), || None, || Some("local".to_string())).is_enabled());
    }

    #[test]
    fn disables_inside_mux_or_ssh() {
        let mut env = kitty_env();
        env.tmux = true;
        assert!(
            !detect_from_env(
                &env,
                || parse_kitty_version("kitty 0.47.4"),
                || Some("local".to_string())
            )
            .is_enabled()
        );

        let mut env = kitty_env();
        env.zellij = true;
        assert!(
            !detect_from_env(
                &env,
                || parse_kitty_version("kitty 0.47.4"),
                || Some("local".to_string())
            )
            .is_enabled()
        );

        let mut env = kitty_env();
        env.ssh_connection = true;
        assert!(
            !detect_from_env(
                &env,
                || parse_kitty_version("kitty 0.47.4"),
                || Some("local".to_string())
            )
            .is_enabled()
        );

        let mut env = kitty_env();
        env.ssh_tty = true;
        assert!(
            !detect_from_env(
                &env,
                || parse_kitty_version("kitty 0.47.4"),
                || Some("local".to_string())
            )
            .is_enabled()
        );
    }

    #[test]
    fn disables_non_kitty_terminals() {
        let env = RuntimeEnv {
            term: Some("xterm-ghostty".to_string()),
            ..RuntimeEnv::default()
        };
        assert!(
            !detect_from_env(
                &env,
                || parse_kitty_version("kitty 0.47.4"),
                || Some("local".to_string())
            )
            .is_enabled()
        );
    }

    #[test]
    fn parses_common_version_outputs() {
        assert_eq!(
            parse_kitty_version("kitty 0.47.4 created by Kovid Goyal"),
            Some(KittyVersion {
                major: 0,
                minor: 47,
                patch: 4
            })
        );
        assert_eq!(
            parse_kitty_version("kitty 0.48.0-alpha"),
            Some(KittyVersion {
                major: 0,
                minor: 48,
                patch: 0
            })
        );
    }
}
