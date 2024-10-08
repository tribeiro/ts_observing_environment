use crate::{error::ObsEnvError, observing_environment::ObservingEnvironment, repos::Repos};
use clap::Parser;
use log;
use std::error::Error;

/// Manage observing environment.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, name = "manage_obs_env")]
pub struct ManageObsEnv {
    /// Which action to execute?
    #[arg(value_enum, long = "action")]
    action: Action,
    /// Log level.
    #[arg(value_enum, long = "log-level", default_value = "debug")]
    log_level: LogLevel,
    /// Path to the environment.
    #[arg(long = "env-path", default_value = "/net/obs-env/auto_base_packages")]
    env_path: String,
    /// Repository to act on (for actions on individual repos).
    #[arg(value_enum, long = "repository")]
    repository: Option<Repos>,
    /// Name of the branch or version to checkout when running the "CheckoutBranch"
    /// or "CheckoutVersion" action.
    #[arg(long = "branch-name", default_value = "")]
    branch_name: String,
    /// Name of the branch to checkout when running the "Reset"
    /// action.
    #[arg(long = "base-env-branch-name", default_value = "main")]
    base_env_branch_name: String,
}
pub trait ManageObsEnvCli {
    fn get_action(&self) -> Result<&Action, Box<dyn Error>>;
    fn get_log_level(&self) -> &LogLevel;
    fn get_env_path(&self) -> &str;
    fn get_branch_name(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_repository_name(&self) -> &str;
    fn get_base_env_source_repo(&self) -> &str;
}

impl ManageObsEnvCli for ManageObsEnv {
    fn get_action(&self) -> Result<&Action, Box<dyn Error>> {
        match self.action {
            Action::CheckoutBranch => {
                if self.repository.is_none() {
                    Err(Box::new(ObsEnvError::ERROR(
                        "Checkout branch action requires a repository, none given".to_owned(),
                    )))
                } else {
                    Ok(&self.action)
                }
            }
            _ => Ok(&self.action),
        }
    }
    fn get_log_level(&self) -> &LogLevel {
        &self.log_level
    }
    fn get_env_path(&self) -> &str {
        &self.env_path
    }
    fn get_branch_name(&self) -> &str {
        &self.branch_name
    }
    fn get_version(&self) -> &str {
        &self.branch_name
    }
    fn get_repository_name(&self) -> &str {
        if let Some(repository) = &self.repository {
            repository.get_name()
        } else {
            ""
        }
    }
    fn get_base_env_source_repo(&self) -> &str {
        &self.base_env_branch_name
    }
}

pub fn run<T>(config: &T) -> Result<(), Box<dyn Error>>
where
    T: ManageObsEnvCli,
{
    match config.get_log_level() {
        LogLevel::Trace => log::set_max_level(log::LevelFilter::Trace),
        LogLevel::Debug => log::set_max_level(log::LevelFilter::Debug),
        LogLevel::Info => log::set_max_level(log::LevelFilter::Info),
        LogLevel::Warn => log::set_max_level(log::LevelFilter::Warn),
        LogLevel::Error => log::set_max_level(log::LevelFilter::Error),
    };

    log::info!("Running manage obs env...");

    let obs_env = ObservingEnvironment::with_destination(config.get_env_path());

    match config.get_action()? {
        Action::Setup => {
            log::info!("Executing Setup...");

            log::debug!("Creating path...");
            obs_env.create_path()?;

            log::debug!("Cloning repositories...");
            let cloned_repos = obs_env.clone_repositories();
            log::info!("The following repositories where cloned: ");
            for repo in cloned_repos.iter() {
                match repo {
                    Ok(repo) => log::info!("{:?}", repo.path()),
                    Err(error) => log::error!("Failed to clone: {error:?}"),
                }
            }
        }
        Action::PrintConfig => {
            log::info!("{}", obs_env.summarize());
        }
        Action::Reset => {
            log::info!("Resetting Observing environment...");
            if let Err(error) = obs_env.reset_base_environment(config.get_base_env_source_repo()) {
                log::error!("Error resetting {} repositories.", error.len());
                for err in error {
                    log::error!("{:?}", err);
                }
            } else {
                log::info!("All repositories set to they base versions.");
            }
        }
        Action::ShowCurrentVersions => {
            log::info!("Current environment versions:");
            let current_versions = obs_env.get_current_env_versions();
            for (name, version) in current_versions.iter() {
                match version {
                    Ok(version) => log::info!("{name}: {version}"),
                    Err(error) => log::error!("{name}: {error:?}"),
                }
            }
        }
        Action::ShowOriginalVersions => {
            match obs_env.get_base_env_versions(config.get_base_env_source_repo()) {
                Ok(base_env_versions) => {
                    log::info!("Base Environment versions:");
                    for (name, version) in base_env_versions.iter() {
                        log::info!("{name}: {version}");
                    }
                }
                Err(error) => {
                    log::error!("{error:?}");
                }
            }
        }
        Action::CheckoutBranch => {
            obs_env.checkout_branch(config.get_repository_name(), config.get_branch_name())?;
        }
        Action::CheckoutVersion => {
            obs_env.reset_index_to_version(config.get_repository_name(), config.get_version())?;
        }
    };
    Ok(())
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Action {
    /// Setup the observing environment?
    /// This will create the destination directory and clone all repositories.
    Setup,
    /// Show observing environment configuration?
    /// This will only print the observing environment configuration.
    PrintConfig,
    /// Reset obs environment. This will bring all repositories in the
    /// environment to their original versions.
    Reset,
    /// Show current versions.
    ShowCurrentVersions,
    /// Show original versions.
    ShowOriginalVersions,
    /// Checkout a branch in a repository.
    CheckoutBranch,
    /// Checkout a version in a repository.
    CheckoutVersion,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
