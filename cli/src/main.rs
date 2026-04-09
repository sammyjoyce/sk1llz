use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime};

const HELP_EXAMPLES: &str = r#"Examples:
  sk1llz catalog list
  sk1llz catalog search rust --format json
  sk1llz catalog show hashimoto-cli-ux
  sk1llz install plan hashimoto-cli-ux
  sk1llz install apply hashimoto-cli-ux --yes
  sk1llz recommend from-text "cli design for ai agents"
  sk1llz recommend from-path .
  sk1llz env doctor
  sk1llz describe install apply

Use 'sk1llz <command> --help' for command-specific details."#;

const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "to", "for", "of", "in", "on", "at", "by", "with", "from", "is",
    "are", "was", "were", "be", "been", "being", "build", "project", "tool", "tools", "using",
    "use", "need", "needs", "want", "wants", "that", "this", "these", "those", "it", "its",
    "their", "them", "your", "into", "over", "under", "through",
];

const SHORT_TOKENS: &[&str] = &["ai", "api", "c", "c++", "ci", "db", "go", "io", "js", "llm", "ml", "qa", "ts", "ui", "ux"];
const PATH_TOKEN_STOP_WORDS: &[&str] = &[
    "docs", "doc", "file", "files", "guide", "guides", "index", "main", "manifest", "meta", "note",
    "notes", "pattern", "patterns", "readme", "reference", "references", "src", "template", "templates",
    "test", "tests", "tmp", "toml", "json", "yaml", "yml", "lock", "troubleshooting", "workflow", "workflows",
];

const PROJECT_SCAN_MAX_DEPTH: usize = 4;
const PROJECT_TOKEN_MAX_DEPTH: usize = 1;
const PROJECT_DOC_SIGNAL_MAX_CHARS: usize = 8_192;
const PROJECT_SCAN_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    "__pycache__",
];
const PROJECT_SIGNAL_IGNORED_DIRS: &[&str] = &[
    "domains",
    "languages",
    "organizations",
    "paradigms",
    "specialists",
    "meta",
];
const PROJECT_DOC_SIGNAL_DIRS: &[&str] = &["cli", "cmd", "tool", "tools", "app", "apps"];

#[derive(Parser, Debug)]
#[command(
    name = "sk1llz",
    author = "copyleftdev",
    version,
    about = "Install and recommend AI coding skills",
    long_about = None,
    subcommand_required = true,
    arg_required_else_help = true,
    propagate_version = true,
    after_help = HELP_EXAMPLES
)]
struct Cli {
    /// Output format. Defaults to text on a TTY and json when piped.
    #[arg(long, global = true, value_enum)]
    format: Option<OutputFormat>,

    /// Shortcut for --format json.
    #[arg(long, global = true, conflicts_with = "format")]
    json: bool,

    /// Disable color output.
    #[arg(long, global = true)]
    no_color: bool,

    /// Reduce non-essential stderr output.
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Increase diagnostic detail on stderr.
    #[arg(short = 'v', long, global = true, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Read and refresh the skill catalog.
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },
    /// Preview or apply skill installation.
    Install {
        #[command(subcommand)]
        command: InstallCommand,
    },
    /// Preview or apply skill removal.
    Remove {
        #[command(subcommand)]
        command: RemoveCommand,
    },
    /// Recommend skills from text or a project path.
    Recommend {
        #[command(subcommand)]
        command: RecommendCommand,
    },
    /// Inspect and prepare the local environment.
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    /// Print machine-readable command metadata.
    Describe(DescribeArgs),
    /// Generate shell completion scripts.
    Completions(CompletionArgs),
}

#[derive(Subcommand, Debug)]
enum CatalogCommand {
    /// List skills from the catalog.
    List(CatalogListArgs),
    /// Search skills by name, id, description, or tags.
    Search(CatalogSearchArgs),
    /// Show one skill in detail.
    Show(CatalogShowArgs),
    /// Refresh the cached manifest from the remote catalog.
    Refresh(RefreshArgs),
}

#[derive(Subcommand, Debug)]
enum InstallCommand {
    /// Preview an install without changing the filesystem.
    Plan(InstallPlanArgs),
    /// Apply an install plan.
    Apply(InstallApplyArgs),
}

#[derive(Subcommand, Debug)]
enum RemoveCommand {
    /// Preview a removal without changing the filesystem.
    Plan(RemovePlanArgs),
    /// Apply a removal plan.
    Apply(RemoveApplyArgs),
}

#[derive(Subcommand, Debug)]
enum RecommendCommand {
    /// Recommend skills from free-form text.
    FromText(RecommendTextArgs),
    /// Recommend skills from a project path.
    FromPath(RecommendPathArgs),
}

#[derive(Subcommand, Debug)]
enum EnvCommand {
    /// Show active install locations.
    Where,
    /// Initialize a project-local .claude/skills directory.
    Init(InitArgs),
    /// Check cache, network, and active install paths.
    Doctor,
}

#[derive(Args, Debug, Clone)]
struct CatalogListArgs {
    /// Filter results by top-level category.
    #[arg(long)]
    category: Option<String>,

    /// Filter results by tag.
    #[arg(long)]
    tag: Option<String>,

    /// Maximum results to return.
    #[arg(long, default_value_t = 25)]
    limit: usize,

    /// Comma-separated fields to include in each item.
    #[arg(long, value_name = "FIELDS")]
    fields: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct CatalogSearchArgs {
    /// Search query.
    query: String,

    /// Filter results by top-level category.
    #[arg(long)]
    category: Option<String>,

    /// Filter results by tag.
    #[arg(long)]
    tag: Option<String>,

    /// Maximum results to return.
    #[arg(long, default_value_t = 15)]
    limit: usize,

    /// Comma-separated fields to include in each item.
    #[arg(long, value_name = "FIELDS")]
    fields: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct CatalogShowArgs {
    /// Skill id or name.
    skill: String,

    /// Comma-separated fields to include in the item.
    #[arg(long, value_name = "FIELDS")]
    fields: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct RefreshArgs {
    /// Show the refresh result without writing the cache.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Debug, Clone)]
struct InstallTargetArgs {
    /// Skill id or name.
    #[arg(value_name = "SKILL", required_unless_present = "request")]
    skill: Option<String>,

    /// Raw JSON request body, or @FILE, or @- for stdin.
    #[arg(long, value_name = "JSON|@FILE|@-", conflicts_with_all = ["skill", "global", "target"])]
    request: Option<String>,

    /// Install into ~/.claude/skills.
    #[arg(long, conflicts_with = "target")]
    global: bool,

    /// Install into a relative path under the current directory.
    #[arg(long, value_name = "PATH", conflicts_with = "global")]
    target: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct InstallPlanArgs {
    #[command(flatten)]
    target: InstallTargetArgs,
}

#[derive(Args, Debug, Clone)]
struct InstallApplyArgs {
    #[command(flatten)]
    target: InstallTargetArgs,

    /// Skip the interactive confirmation prompt.
    #[arg(long)]
    yes: bool,

    /// Print the apply result without writing files.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Debug, Clone)]
struct RemoveTargetArgs {
    /// Skill id or name.
    #[arg(value_name = "SKILL", required_unless_present = "request")]
    skill: Option<String>,

    /// Raw JSON request body, or @FILE, or @- for stdin.
    #[arg(long, value_name = "JSON|@FILE|@-", conflicts_with_all = ["skill", "global"])]
    request: Option<String>,

    /// Remove from ~/.claude/skills.
    #[arg(long)]
    global: bool,
}

#[derive(Args, Debug, Clone)]
struct RemovePlanArgs {
    #[command(flatten)]
    target: RemoveTargetArgs,
}

#[derive(Args, Debug, Clone)]
struct RemoveApplyArgs {
    #[command(flatten)]
    target: RemoveTargetArgs,

    /// Skip the interactive confirmation prompt.
    #[arg(long)]
    yes: bool,

    /// Print the apply result without deleting files.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Debug, Clone)]
struct RecommendReadArgs {
    /// Maximum results to return.
    #[arg(long, default_value_t = 10)]
    limit: usize,

    /// Comma-separated fields to include in each recommendation.
    #[arg(long, value_name = "FIELDS")]
    fields: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct RecommendTextArgs {
    /// Free-form description. If omitted and stdin is piped, stdin is used.
    description: Option<String>,

    /// Read the description from stdin.
    #[arg(long)]
    stdin: bool,

    #[command(flatten)]
    read: RecommendReadArgs,
}

#[derive(Args, Debug, Clone)]
struct RecommendPathArgs {
    /// Project path. Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    #[command(flatten)]
    read: RecommendReadArgs,
}

#[derive(Args, Debug, Clone)]
struct InitArgs {
    /// Show the init result without creating directories.
    #[arg(long)]
    dry_run: bool,
}

#[derive(Args, Debug, Clone)]
struct DescribeArgs {
    /// Optional command path, for example: install apply
    path: Vec<String>,
}

#[derive(Args, Debug, Clone)]
struct CompletionArgs {
    /// Target shell.
    #[arg(value_enum)]
    shell: Shell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Manifest {
    version: String,
    generated_at: String,
    repository: String,
    raw_base_url: String,
    skill_count: usize,
    skills: Vec<Skill>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Skill {
    id: String,
    name: String,
    description: String,
    category: String,
    subcategory: Option<String>,
    path: String,
    files: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ErrorPayload<'a> {
    error: ErrorBody<'a>,
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    kind: &'a str,
    message: &'a str,
    hint: Option<&'a str>,
    details: Option<&'a Value>,
}

#[derive(Debug, Clone)]
struct CliError {
    exit_code: i32,
    kind: &'static str,
    message: String,
    hint: Option<String>,
    details: Option<Value>,
}

impl CliError {
    fn usage(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            exit_code: 1,
            kind: "usage",
            message: message.into(),
            hint: Some(hint.into()),
            details: None,
        }
    }

    fn internal(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            exit_code: 1,
            kind: "internal",
            message: message.into(),
            hint: Some(hint.into()),
            details: None,
        }
    }

    fn not_found(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            exit_code: 3,
            kind: "not_found",
            message: message.into(),
            hint: Some(hint.into()),
            details: None,
        }
    }

    fn remote(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            exit_code: 4,
            kind: "remote",
            message: message.into(),
            hint: Some(hint.into()),
            details: None,
        }
    }

    fn emit(&self, ctx: &AppContext) {
        if ctx.is_json() {
            let payload = ErrorPayload {
                error: ErrorBody {
                    kind: self.kind,
                    message: &self.message,
                    hint: self.hint.as_deref(),
                    details: self.details.as_ref(),
                },
            };
            match serde_json::to_string_pretty(&payload) {
                Ok(encoded) => eprintln!("{encoded}"),
                Err(_) => eprintln!("{{\"error\":{{\"kind\":\"internal\",\"message\":\"failed to encode error\"}}}}"),
            }
            return;
        }

        eprintln!("{} {}", "Error:".red().bold(), self.message);
        if let Some(hint) = &self.hint {
            eprintln!("{} {}", "Fix:".green().bold(), hint);
        }
        if ctx.verbose > 0 {
            if let Some(details) = &self.details {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(details)
                        .unwrap_or_else(|_| details.to_string())
                        .dimmed()
                );
            }
        }
    }
}

type AppResult<T> = Result<T, CliError>;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Scope {
    Project,
    Global,
    Custom,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Scope::Project => "project",
            Scope::Global => "global",
            Scope::Custom => "custom",
        }
    }
}

#[derive(Debug)]
struct AppContext {
    format: OutputFormat,
    quiet: bool,
    verbose: u8,
    color_enabled: bool,
    explicit_json: bool,
}

impl AppContext {
    fn from_cli(cli: &Cli) -> Self {
        let explicit_json = cli.json || matches!(cli.format, Some(OutputFormat::Json));
        let format = if cli.json {
            OutputFormat::Json
        } else if let Some(format) = cli.format {
            format
        } else if io::stdout().is_terminal() {
            OutputFormat::Text
        } else {
            OutputFormat::Json
        };

        let no_color_env = std::env::var_os("NO_COLOR").is_some();
        let dumb_term = std::env::var("TERM")
            .map(|value| value == "dumb")
            .unwrap_or(false);
        let color_enabled = !cli.no_color
            && !no_color_env
            && !dumb_term
            && io::stderr().is_terminal()
            && format == OutputFormat::Text;

        Self {
            format,
            quiet: cli.quiet,
            verbose: cli.verbose,
            color_enabled,
            explicit_json,
        }
    }

    fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }

    fn explicit_json_requested(&self) -> bool {
        self.explicit_json
    }
}

#[derive(Debug)]
struct Response {
    value: Value,
    text: String,
}

#[derive(Debug)]
struct Outcome {
    exit_code: i32,
    response: Response,
}

impl Outcome {
    fn ok(value: Value, text: String) -> Self {
        Self {
            exit_code: 0,
            response: Response { value, text },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallRequest {
    skill: String,
    #[serde(default)]
    global: bool,
    target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoveRequest {
    skill: String,
    #[serde(default)]
    global: bool,
}

#[derive(Debug)]
struct EnvironmentPaths {
    project_root: Option<PathBuf>,
    project_skills_dir: Option<PathBuf>,
    global_skills_dir: PathBuf,
}

#[derive(Debug)]
struct InstallPlanData {
    request: InstallRequest,
    skill: Skill,
    scope: Scope,
    target_dir: PathBuf,
    file_plans: Vec<FilePlan>,
}

#[derive(Debug, Clone, Serialize)]
struct FilePlan {
    file: String,
    source_url: String,
    destination: String,
    status: String,
}

#[derive(Debug)]
struct RemovePlanData {
    request: RemoveRequest,
    scope: Scope,
    target_dir: PathBuf,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    name: String,
    status: String,
    detail: String,
    fix: Option<String>,
}

#[derive(Debug)]
struct ProjectAnalysis {
    total_files: usize,
    frameworks: Vec<String>,
    config_files: Vec<String>,
    extension_counts: Vec<(String, usize)>,
    tokens: Vec<String>,
    signals: Vec<RecommendSignal>,
}

#[derive(Debug, Clone)]
struct Recommendation {
    skill: Skill,
    score: i64,
    reasons: Vec<String>,
    score_breakdown: ScoreBreakdown,
    matched_signals: usize,
    strong_matches: usize,
}

#[derive(Debug, Clone)]
struct RecommendSignal {
    kind: &'static str,
    value: String,
    weight: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ScoreComponent {
    feature: String,
    signal: String,
    weight: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ScoreBreakdown {
    total: i64,
    components: Vec<ScoreComponent>,
}

#[derive(Debug, Clone)]
struct SkillProfile {
    tag_terms: HashSet<String>,
    scope_terms: HashSet<String>,
    name_terms: HashSet<String>,
    id_terms: HashSet<String>,
    description_terms: HashSet<String>,
}

fn manifest_url() -> String {
    std::env::var("SKILLZ_MANIFEST_URL").unwrap_or_else(|_| {
        "https://raw.githubusercontent.com/copyleftdev/sk1llz/master/skills.json".to_string()
    })
}

fn raw_base_url() -> String {
    std::env::var("SKILLZ_RAW_BASE_URL").unwrap_or_else(|_| {
        "https://raw.githubusercontent.com/copyleftdev/sk1llz/master".to_string()
    })
}

fn cache_dir() -> AppResult<PathBuf> {
    let path = dirs::cache_dir()
        .ok_or_else(|| {
            CliError::internal(
                "could not determine the cache directory",
                "set XDG_CACHE_HOME or HOME and try again",
            )
        })?
        .join("sk1llz");
    fs::create_dir_all(&path).map_err(|error| {
        CliError::internal(
            format!("failed to create cache directory at {}", path.display()),
            error.to_string(),
        )
    })?;
    Ok(path)
}

fn manifest_cache_path() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("skills.json"))
}

fn http_client() -> AppResult<Client> {
    Client::builder()
        .user_agent(format!("sk1llz/{}", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| {
            CliError::internal("failed to initialize the HTTP client", error.to_string())
        })
}

fn spinner(ctx: &AppContext, message: &str) -> Option<ProgressBar> {
    if ctx.quiet || ctx.is_json() || !io::stderr().is_terminal() {
        return None;
    }

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    progress.set_message(message.to_string());
    progress.enable_steady_tick(Duration::from_millis(80));
    Some(progress)
}

fn progress_bar(ctx: &AppContext, len: u64, message: &str) -> Option<ProgressBar> {
    if ctx.quiet || ctx.is_json() || !io::stderr().is_terminal() {
        return None;
    }

    let progress = ProgressBar::new(len);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );
    progress.set_message(message.to_string());
    Some(progress)
}

fn emit_outcome(ctx: &AppContext, outcome: Outcome) -> AppResult<()> {
    if ctx.is_json() {
        if outcome.response.value.is_null() && outcome.response.text.is_empty() {
            return Ok(());
        }
        let encoded = serde_json::to_string_pretty(&outcome.response.value).map_err(|error| {
            CliError::internal("failed to encode command output", error.to_string())
        })?;
        println!("{encoded}");
        return Ok(());
    }

    if !outcome.response.text.is_empty() {
        println!("{}", outcome.response.text.trim_end());
    }

    Ok(())
}

fn environment_paths() -> AppResult<EnvironmentPaths> {
    let cwd = std::env::current_dir().map_err(|error| {
        CliError::internal(
            "could not determine the current directory",
            error.to_string(),
        )
    })?;
    let project_root = find_repo_root(&cwd).or_else(|| {
        let claude_dir = cwd.join(".claude");
        if claude_dir.exists() && claude_dir.is_dir() {
            Some(cwd.clone())
        } else {
            None
        }
    });
    let project_skills_dir = project_root.as_ref().and_then(|root| {
        let claude_dir = root.join(".claude");
        if claude_dir.exists() && claude_dir.is_dir() {
            Some(claude_dir.join("skills"))
        } else {
            None
        }
    });
    let global_skills_dir = dirs::home_dir()
        .ok_or_else(|| {
            CliError::internal(
                "could not determine the home directory",
                "set HOME and try again",
            )
        })?
        .join(".claude")
        .join("skills");

    Ok(EnvironmentPaths {
        project_root,
        project_skills_dir,
        global_skills_dir,
    })
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    for candidate in start.ancestors() {
        let git_dir = candidate.join(".git");
        if git_dir.exists() {
            return Some(candidate.to_path_buf());
        }
    }
    None
}

fn manifest_age_days() -> AppResult<u64> {
    let metadata = fs::metadata(manifest_cache_path()?).map_err(|error| {
        CliError::internal(
            "failed to read the manifest cache metadata",
            error.to_string(),
        )
    })?;
    let modified = metadata.modified().map_err(|error| {
        CliError::internal(
            "failed to read the manifest cache timestamp",
            error.to_string(),
        )
    })?;
    let age = SystemTime::now()
        .duration_since(modified)
        .map_err(|error| {
            CliError::internal(
                "failed to calculate the manifest cache age",
                error.to_string(),
            )
        })?;
    Ok(age.as_secs() / 86_400)
}

fn fetch_manifest(ctx: &AppContext, dry_run: bool) -> AppResult<Manifest> {
    let progress = spinner(ctx, "Fetching the skill catalog...");
    let response = http_client()?.get(manifest_url()).send().map_err(|error| {
        CliError::remote("failed to fetch the skill catalog", error.to_string())
    })?;

    let status = response.status();
    if !status.is_success() {
        if let Some(progress) = progress {
            progress.finish_and_clear();
        }
        return Err(CliError::remote(
            format!("catalog request failed with HTTP {}", status.as_u16()),
            "check your network connection or SKILLZ_MANIFEST_URL and try again",
        ));
    }

    let manifest = response.json::<Manifest>().map_err(|error| {
        CliError::remote("failed to decode the skill catalog", error.to_string())
    })?;

    if let Some(progress) = progress {
        if dry_run {
            progress.finish_with_message("Fetched manifest (dry run)");
        } else {
            progress.finish_with_message(format!("Fetched {} skills", manifest.skill_count));
        }
    }

    if !dry_run {
        let cache_path = manifest_cache_path()?;
        let encoded = serde_json::to_string_pretty(&manifest).map_err(|error| {
            CliError::internal("failed to encode the manifest cache", error.to_string())
        })?;
        fs::write(&cache_path, encoded).map_err(|error| {
            CliError::internal(
                format!(
                    "failed to write the manifest cache to {}",
                    cache_path.display()
                ),
                error.to_string(),
            )
        })?;
    }

    Ok(manifest)
}

fn load_manifest(ctx: &AppContext) -> AppResult<Manifest> {
    let cache_path = manifest_cache_path()?;
    if !cache_path.exists() {
        return fetch_manifest(ctx, false);
    }

    let content = fs::read_to_string(&cache_path).map_err(|error| {
        CliError::internal(
            format!(
                "failed to read the cached manifest at {}",
                cache_path.display()
            ),
            error.to_string(),
        )
    })?;

    serde_json::from_str(&content).map_err(|error| {
        CliError::internal(
            "the cached manifest is invalid",
            format!("run 'sk1llz catalog refresh' to replace the cache ({error})"),
        )
    })
}

fn validate_limit(limit: usize) -> AppResult<()> {
    if limit == 0 {
        return Err(CliError::usage(
            "--limit must be greater than zero",
            "pick a positive limit such as --limit 10",
        ));
    }
    Ok(())
}

fn parse_fields(fields: &Option<String>) -> AppResult<Option<Vec<String>>> {
    let Some(fields) = fields else {
        return Ok(None);
    };

    let mut parsed = Vec::new();
    for field in fields.split(',') {
        let trimmed = field.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            return Err(CliError::usage(
                format!("invalid field selector '{trimmed}'"),
                "use comma-separated field names such as --fields id,name,description",
            ));
        }
        parsed.push(trimmed.to_string());
    }

    if parsed.is_empty() {
        return Err(CliError::usage(
            "the field list was empty",
            "use comma-separated field names such as --fields id,name",
        ));
    }

    Ok(Some(parsed))
}

fn filter_object_fields(value: &Value, fields: &[String]) -> Value {
    let mut filtered = serde_json::Map::new();
    if let Some(object) = value.as_object() {
        for field in fields {
            if let Some(entry) = object.get(field) {
                filtered.insert(field.clone(), entry.clone());
            }
        }
    }
    Value::Object(filtered)
}

fn filter_recommendation_fields(value: &Value, fields: &[String]) -> Value {
    let mut filtered = serde_json::Map::new();
    let Some(object) = value.as_object() else {
        return Value::Object(filtered);
    };
    let skill = object.get("skill").and_then(Value::as_object);

    for field in fields {
        if let Some(entry) = object.get(field) {
            filtered.insert(field.clone(), entry.clone());
            continue;
        }

        if let Some(entry) = skill.and_then(|skill| skill.get(field)) {
            filtered.insert(field.clone(), entry.clone());
        }
    }

    Value::Object(filtered)
}

fn render_filtered_text(value: &Value) -> String {
    match value {
        Value::Array(items) => items
            .iter()
            .map(render_filtered_text)
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(object) => object
            .iter()
            .map(|(key, value)| format!("{key}: {}", compact_json(value)))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => compact_json(value),
    }
}

fn compact_json(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "<invalid-json>".to_string()),
    }
}

fn validate_skill_ref(value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CliError::usage(
            "skill id cannot be empty",
            "pass a skill id such as hashimoto-cli-ux",
        ));
    }
    if trimmed != value {
        return Err(CliError::usage(
            "skill id cannot contain leading or trailing whitespace",
            "remove the surrounding whitespace and try again",
        ));
    }
    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('?')
        || trimmed.contains('#')
        || trimmed.contains('%')
    {
        return Err(CliError::usage(
            format!("skill id '{trimmed}' contains reserved path characters"),
            "use a bare skill id such as hashimoto-cli-ux",
        ));
    }
    if trimmed.contains("..") {
        return Err(CliError::usage(
            format!("skill id '{trimmed}' contains a parent-directory segment"),
            "use a bare skill id such as hashimoto-cli-ux",
        ));
    }
    if trimmed.chars().any(|character| character.is_control()) {
        return Err(CliError::usage(
            "skill id contains control characters",
            "remove the control characters and try again",
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_target_path(value: &str) -> AppResult<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CliError::usage(
            "--target cannot be empty",
            "use a relative path such as --target ./.claude/skills-local",
        ));
    }
    if Path::new(trimmed).is_absolute() {
        return Err(CliError::usage(
            "--target must be a relative path under the current directory",
            "use --global for ~/.claude/skills or pass a relative path such as --target ./skills",
        ));
    }
    if trimmed.contains('?') || trimmed.contains('#') || trimmed.contains('%') {
        return Err(CliError::usage(
            "--target cannot contain reserved URL characters",
            "use a plain relative filesystem path",
        ));
    }

    let path = Path::new(trimmed);
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                if segment
                    .to_string_lossy()
                    .chars()
                    .any(|character| character.is_control())
                {
                    return Err(CliError::usage(
                        "--target cannot contain control characters",
                        "use a plain relative filesystem path",
                    ));
                }
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CliError::usage(
                    "--target must stay under the current directory",
                    "use a relative path without '..' segments",
                ));
            }
        }
    }

    let cwd = std::env::current_dir().map_err(|error| {
        CliError::internal(
            "could not determine the current directory",
            error.to_string(),
        )
    })?;
    Ok(cwd.join(path))
}

fn validate_install_scope(global: bool, target: Option<&str>) -> AppResult<()> {
    if global && target.is_some() {
        return Err(CliError::usage(
            "--global and --target cannot be used together",
            "pick either --global or a relative --target path",
        ));
    }
    Ok(())
}

fn safe_relative_file_path(relative: &str) -> AppResult<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute() {
        return Err(CliError::internal(
            format!("manifest file path '{relative}' is absolute"),
            "check the manifest generator; file entries must stay relative",
        ));
    }

    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                let segment_text = segment.to_string_lossy();
                if segment_text.chars().any(|character| character.is_control()) {
                    return Err(CliError::internal(
                        format!("manifest file path '{relative}' contains control characters"),
                        "check the manifest generator; file entries must be plain relative paths",
                    ));
                }
                clean.push(segment);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CliError::internal(
                    format!("manifest file path '{relative}' escapes the skill directory"),
                    "check the manifest generator; file entries must be plain relative paths",
                ));
            }
        }
    }

    if clean.as_os_str().is_empty() {
        return Err(CliError::internal(
            format!("manifest file path '{relative}' is empty"),
            "check the manifest generator; file entries must be plain relative paths",
        ));
    }

    Ok(clean)
}

fn read_json_input(value: &str) -> AppResult<String> {
    if value == "@-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).map_err(|error| {
            CliError::usage(
                "failed to read the request body from stdin",
                error.to_string(),
            )
        })?;
        if input.trim().is_empty() {
            return Err(CliError::usage(
                "stdin did not contain a request body",
                "pipe a JSON object into the command or pass --request @FILE",
            ));
        }
        return Ok(input);
    }

    if let Some(path) = value.strip_prefix('@') {
        let content = fs::read_to_string(path).map_err(|error| {
            CliError::usage(
                format!("failed to read request file '{path}'"),
                error.to_string(),
            )
        })?;
        if content.trim().is_empty() {
            return Err(CliError::usage(
                format!("request file '{path}' was empty"),
                "put a JSON object in the file or pass an inline JSON object",
            ));
        }
        return Ok(content);
    }

    Ok(value.to_string())
}

fn read_stdin_text(empty_hint: &str) -> AppResult<String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| CliError::usage("failed to read from stdin", error.to_string()))?;
    if input.trim().is_empty() {
        return Err(CliError::usage("stdin was empty", empty_hint.to_string()));
    }
    Ok(input)
}

fn resolve_install_request(target: &InstallTargetArgs) -> AppResult<InstallRequest> {
    if let Some(raw) = &target.request {
        let decoded = read_json_input(raw)?;
        let request = serde_json::from_str::<InstallRequest>(&decoded).map_err(|error| {
            CliError::usage(
                "failed to parse the install request JSON",
                format!("pass a JSON object such as {{\"skill\":\"hashimoto-cli-ux\"}} ({error})"),
            )
        })?;

        let skill = validate_skill_ref(&request.skill)?;
        validate_install_scope(request.global, request.target.as_deref())?;
        if let Some(path) = &request.target {
            let _ = validate_target_path(path)?;
        }

        return Ok(InstallRequest {
            skill,
            global: request.global,
            target: request.target,
        });
    }

    let skill = validate_skill_ref(target.skill.as_deref().unwrap_or_default())?;
    validate_install_scope(target.global, target.target.as_deref())?;
    if let Some(path) = &target.target {
        let _ = validate_target_path(path)?;
    }

    Ok(InstallRequest {
        skill,
        global: target.global,
        target: target.target.clone(),
    })
}

fn resolve_remove_request(target: &RemoveTargetArgs) -> AppResult<RemoveRequest> {
    if let Some(raw) = &target.request {
        let decoded = read_json_input(raw)?;
        let request = serde_json::from_str::<RemoveRequest>(&decoded).map_err(|error| {
            CliError::usage(
                "failed to parse the remove request JSON",
                format!("pass a JSON object such as {{\"skill\":\"hashimoto-cli-ux\"}} ({error})"),
            )
        })?;

        return Ok(RemoveRequest {
            skill: validate_skill_ref(&request.skill)?,
            global: request.global,
        });
    }

    Ok(RemoveRequest {
        skill: validate_skill_ref(target.skill.as_deref().unwrap_or_default())?,
        global: target.global,
    })
}

fn count_installed_skills(path: &Path) -> usize {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_dir())
                .count()
        })
        .unwrap_or(0)
}

fn installed_paths_for_skill(paths: &EnvironmentPaths, skill_id: &str) -> Vec<String> {
    let mut installed = Vec::new();

    if let Some(project_path) = &paths.project_skills_dir {
        let candidate = project_path.join(skill_id);
        if candidate.exists() {
            installed.push(candidate.display().to_string());
        }
    }

    let global_candidate = paths.global_skills_dir.join(skill_id);
    if global_candidate.exists() {
        installed.push(global_candidate.display().to_string());
    }

    installed
}

fn find_skill<'a>(manifest: &'a Manifest, value: &str) -> AppResult<&'a Skill> {
    let needle = value.to_lowercase();
    if let Some(skill) = manifest.skills.iter().find(|skill| {
        skill.id.eq_ignore_ascii_case(&needle) || skill.name.eq_ignore_ascii_case(&needle)
    }) {
        return Ok(skill);
    }

    let suggestions = find_similar_skills(value, &manifest.skills);
    let hint = if suggestions.is_empty() {
        "run 'sk1llz catalog list' to inspect available skills".to_string()
    } else {
        format!(
            "did you mean {}? Run 'sk1llz catalog show <skill>' for details.",
            suggestions.join(", ")
        )
    };

    Err(CliError::not_found(
        format!("skill '{value}' was not found"),
        hint,
    ))
}

fn find_similar_skills(query: &str, skills: &[Skill]) -> Vec<String> {
    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(String, i64)> = skills
        .iter()
        .filter_map(|skill| {
            let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
            let id_score = matcher.fuzzy_match(&skill.id, query).unwrap_or(0);
            let score = name_score.max(id_score);
            if score > 20 {
                Some((skill.id.clone(), score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|left, right| right.1.cmp(&left.1));
    scored.into_iter().take(3).map(|(id, _)| id).collect()
}

fn skill_summary_json(skill: &Skill, installed_paths: Vec<String>) -> Value {
    json!({
        "id": skill.id,
        "name": skill.name,
        "category": skill.category,
        "subcategory": skill.subcategory,
        "description": skill.description,
        "tags": skill.tags,
        "installed": !installed_paths.is_empty(),
        "installed_paths": installed_paths,
    })
}

fn skill_detail_json(skill: &Skill, manifest: &Manifest, installed_paths: Vec<String>) -> Value {
    json!({
        "id": skill.id,
        "name": skill.name,
        "category": skill.category,
        "subcategory": skill.subcategory,
        "description": skill.description,
        "tags": skill.tags,
        "files": skill.files,
        "repository_path": skill.path,
        "repository_url": format!("{}/{}", manifest.repository, skill.path),
        "installed": !installed_paths.is_empty(),
        "installed_paths": installed_paths,
    })
}

fn build_install_plan(
    manifest: &Manifest,
    request: InstallRequest,
    paths: &EnvironmentPaths,
) -> AppResult<InstallPlanData> {
    validate_install_scope(request.global, request.target.as_deref())?;
    let skill = find_skill(manifest, &request.skill)?.clone();
    let (scope, target_dir) = if let Some(target) = &request.target {
        (Scope::Custom, validate_target_path(target)?.join(&skill.id))
    } else if request.global {
        (Scope::Global, paths.global_skills_dir.join(&skill.id))
    } else if let Some(project_path) = &paths.project_skills_dir {
        (Scope::Project, project_path.join(&skill.id))
    } else {
        (Scope::Global, paths.global_skills_dir.join(&skill.id))
    };

    let mut file_plans = Vec::new();
    for file in &skill.files {
        let relative = safe_relative_file_path(file)?;
        let destination = target_dir.join(&relative);
        let status = if destination.exists() {
            "replace"
        } else {
            "create"
        }
        .to_string();
        file_plans.push(FilePlan {
            file: relative.display().to_string(),
            source_url: format!("{}/{}/{}", raw_base_url(), skill.path, file),
            destination: destination.display().to_string(),
            status,
        });
    }

    Ok(InstallPlanData {
        request,
        skill,
        scope,
        target_dir,
        file_plans,
    })
}

fn build_remove_plan(
    manifest: &Manifest,
    request: RemoveRequest,
    paths: &EnvironmentPaths,
) -> AppResult<RemovePlanData> {
    let skill = find_skill(manifest, &request.skill)?.clone();
    let (scope, target_dir) = if request.global {
        (Scope::Global, paths.global_skills_dir.join(&skill.id))
    } else if let Some(project_path) = &paths.project_skills_dir {
        (Scope::Project, project_path.join(&skill.id))
    } else {
        (Scope::Global, paths.global_skills_dir.join(&skill.id))
    };

    if !target_dir.exists() {
        return Err(CliError::not_found(
            format!(
                "skill '{}' is not installed in the {} scope",
                skill.id,
                scope.as_str()
            ),
            "run 'sk1llz env where' to inspect active install locations",
        ));
    }

    Ok(RemovePlanData {
        request,
        scope,
        target_dir,
        status: "delete",
    })
}

fn confirm_apply(ctx: &AppContext, prompt: &str, yes: bool) -> AppResult<()> {
    if yes {
        return Ok(());
    }

    if !io::stdin().is_terminal() {
        return Err(CliError::usage(
            "refusing to prompt because stdin is not a TTY",
            "re-run with --yes in non-interactive environments",
        ));
    }

    if !ctx.quiet {
        eprint!("{} {} [y/N]: ", "Confirm:".yellow().bold(), prompt);
        io::stderr().flush().map_err(|error| {
            CliError::internal("failed to flush the confirmation prompt", error.to_string())
        })?;
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|error| {
        CliError::internal(
            "failed to read the confirmation response",
            error.to_string(),
        )
    })?;

    let answer = input.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        Err(CliError::usage(
            "operation cancelled",
            "re-run with --yes to skip the confirmation prompt",
        ))
    }
}

fn apply_install_plan(ctx: &AppContext, plan: &InstallPlanData) -> AppResult<()> {
    fs::create_dir_all(&plan.target_dir).map_err(|error| {
        CliError::internal(
            format!("failed to create {}", plan.target_dir.display()),
            error.to_string(),
        )
    })?;

    let progress = progress_bar(ctx, plan.file_plans.len() as u64, "Installing files");
    let client = http_client()?;

    for file in &plan.file_plans {
        let destination = PathBuf::from(&file.destination);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                CliError::internal(
                    format!("failed to create {}", parent.display()),
                    error.to_string(),
                )
            })?;
        }

        let response = client.get(&file.source_url).send().map_err(|error| {
            CliError::remote(format!("failed to fetch {}", file.file), error.to_string())
        })?;

        let status = response.status();
        if !status.is_success() {
            if let Some(progress) = &progress {
                progress.finish_and_clear();
            }
            return Err(CliError::remote(
                format!("failed to fetch {} (HTTP {})", file.file, status.as_u16()),
                "check the remote catalog path and try again",
            ));
        }

        let bytes = response.bytes().map_err(|error| {
            CliError::remote(format!("failed to read {}", file.file), error.to_string())
        })?;
        fs::write(&destination, &bytes).map_err(|error| {
            CliError::internal(
                format!("failed to write {}", destination.display()),
                error.to_string(),
            )
        })?;

        if let Some(progress) = &progress {
            progress.set_message(file.file.clone());
            progress.inc(1);
        }
    }

    if let Some(progress) = progress {
        progress.finish_with_message("Install complete");
    }

    Ok(())
}

fn apply_remove_plan(plan: &RemovePlanData) -> AppResult<()> {
    fs::remove_dir_all(&plan.target_dir).map_err(|error| {
        CliError::internal(
            format!("failed to remove {}", plan.target_dir.display()),
            error.to_string(),
        )
    })
}

fn analyze_project(path: &Path) -> AppResult<ProjectAnalysis> {
    if !path.exists() {
        return Err(CliError::not_found(
            format!("path '{}' does not exist", path.display()),
            "pass a valid project path or omit the path to analyze the current directory",
        ));
    }

    let mut total_files = 0usize;
    let mut extension_counts: HashMap<String, usize> = HashMap::new();
    let mut frameworks = HashSet::new();
    let mut config_files = HashSet::new();
    let mut keyword_counts: HashMap<String, usize> = HashMap::new();

    scan_directory(
        path,
        0,
        false,
        &mut total_files,
        &mut extension_counts,
        &mut frameworks,
        &mut config_files,
        &mut keyword_counts,
    )?;
    ingest_project_doc_signals(path, &mut keyword_counts);

    let mut frameworks: Vec<String> = frameworks.into_iter().collect();
    frameworks.sort();

    let mut config_files: Vec<String> = config_files.into_iter().collect();
    config_files.sort();

    let signals = build_project_signals(&frameworks, &config_files, &keyword_counts, &extension_counts);

    let mut extension_counts: Vec<(String, usize)> = extension_counts.into_iter().collect();
    extension_counts.sort_by(|left, right| right.1.cmp(&left.1));

    let mut tokens: Vec<String> = signals.iter().map(|signal| signal.value.clone()).collect();
    tokens.sort();
    tokens.dedup();

    Ok(ProjectAnalysis {
        total_files,
        frameworks,
        config_files,
        extension_counts,
        tokens,
        signals,
    })
}

fn scan_directory(
    path: &Path,
    depth: usize,
    suppress_keywords: bool,
    total_files: &mut usize,
    extension_counts: &mut HashMap<String, usize>,
    frameworks: &mut HashSet<String>,
    config_files: &mut HashSet<String>,
    keyword_counts: &mut HashMap<String, usize>,
) -> AppResult<()> {
    if depth > PROJECT_SCAN_MAX_DEPTH {
        return Ok(());
    }

    let entries = fs::read_dir(path).map_err(|error| {
        CliError::internal(
            format!("failed to read {}", path.display()),
            error.to_string(),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            CliError::internal("failed to read a directory entry", error.to_string())
        })?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if entry_path.is_dir() {
            if file_name.starts_with('.')
                || PROJECT_SCAN_IGNORED_DIRS
                    .iter()
                    .any(|candidate| candidate == &file_name.as_ref())
            {
                continue;
            }

            let suppress_entry_keywords = suppress_keywords
                || PROJECT_SIGNAL_IGNORED_DIRS
                    .iter()
                    .any(|candidate| candidate == &file_name.as_ref());
            if !suppress_entry_keywords {
                ingest_directory_signal(file_name.as_ref(), depth, keyword_counts);
            }
            scan_directory(
                &entry_path,
                depth + 1,
                suppress_entry_keywords,
                total_files,
                extension_counts,
                frameworks,
                config_files,
                keyword_counts,
            )?;
            continue;
        }

        *total_files += 1;
        ingest_file_signal(
            &entry_path,
            &file_name,
            depth,
            suppress_keywords,
            extension_counts,
            frameworks,
            config_files,
            keyword_counts,
        );
    }

    Ok(())
}

fn ingest_directory_signal(name: &str, depth: usize, keyword_counts: &mut HashMap<String, usize>) {
    if depth > PROJECT_TOKEN_MAX_DEPTH {
        return;
    }

    for token in tokenize(name) {
        if should_keep_path_token(&token) {
            *keyword_counts.entry(token).or_insert(0) += 1;
        }
    }
}

fn ingest_file_signal(
    path: &Path,
    file_name: &str,
    depth: usize,
    suppress_keywords: bool,
    extension_counts: &mut HashMap<String, usize>,
    frameworks: &mut HashSet<String>,
    config_files: &mut HashSet<String>,
    keyword_counts: &mut HashMap<String, usize>,
) {
    if let Some(extension) = path.extension().and_then(|value| value.to_str()) {
        let extension = extension.to_ascii_lowercase();
        *extension_counts.entry(extension.clone()).or_insert(0) += 1;

        match extension.as_str() {
            "rs" => {
                frameworks.insert("rust".to_string());
            }
            "py" => {
                frameworks.insert("python".to_string());
            }
            "go" => {
                frameworks.insert("go".to_string());
            }
            "zig" => {
                frameworks.insert("zig".to_string());
            }
            "ts" | "tsx" => {
                frameworks.insert("typescript".to_string());
                frameworks.insert("javascript".to_string());
            }
            "js" | "jsx" => {
                frameworks.insert("javascript".to_string());
            }
            "c" | "h" => {
                frameworks.insert("c".to_string());
            }
            "cc" | "cpp" | "cxx" | "hpp" => {
                frameworks.insert("cpp".to_string());
            }
            _ => {}
        }
    }

    let lower = file_name.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "cargo.toml"
            | "package.json"
            | "go.mod"
            | "flake.nix"
            | "shell.nix"
            | "dockerfile"
            | "compose.yaml"
            | "compose.yml"
    ) {
        config_files.insert(file_name.to_string());
    }

    match lower.as_str() {
        "cargo.toml" => {
            frameworks.insert("rust".to_string());
        }
        "package.json" => {
            frameworks.insert("javascript".to_string());
            frameworks.insert("typescript".to_string());
        }
        "go.mod" => {
            frameworks.insert("go".to_string());
        }
        _ => {}
    }

    if !suppress_keywords && depth <= PROJECT_TOKEN_MAX_DEPTH {
        for token in tokenize(file_name) {
            if should_keep_path_token(&token) {
                *keyword_counts.entry(token).or_insert(0) += 1;
            }
        }
    }
}

fn ingest_project_doc_signals(root: &Path, keyword_counts: &mut HashMap<String, usize>) {
    let mut candidates = vec![root.join("README.md"), root.join("README")];
    for dir in PROJECT_DOC_SIGNAL_DIRS {
        candidates.push(root.join(dir).join("README.md"));
    }

    for candidate in candidates {
        let Ok(metadata) = fs::metadata(&candidate) else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        let Ok(content) = fs::read_to_string(&candidate) else {
            continue;
        };
        let sample: String = content.chars().take(PROJECT_DOC_SIGNAL_MAX_CHARS).collect();
        ingest_doc_signal_content(&sample, keyword_counts);
    }
}

fn boost_signal_from_markers(
    keyword_counts: &mut HashMap<String, usize>,
    signal: &str,
    haystack: &str,
    markers: &[&str],
    max_hits: usize,
) {
    let hits = markers
        .iter()
        .map(|marker| haystack.matches(marker).count())
        .sum::<usize>()
        .min(max_hits);

    if hits > 0 {
        *keyword_counts.entry(signal.to_string()).or_insert(0) += hits;
    }
}

fn ingest_doc_signal_content(content: &str, keyword_counts: &mut HashMap<String, usize>) {
    let lower = content.to_ascii_lowercase();

    boost_signal_from_markers(
        keyword_counts,
        "cli",
        &lower,
        &["cli", "command-line", "subcommand", "command tree", "output contract"],
        4,
    );
    boost_signal_from_markers(
        keyword_counts,
        "automation",
        &lower,
        &["--json", "machine-readable", "stdout", "stderr", "dry-run"],
        3,
    );
    boost_signal_from_markers(
        keyword_counts,
        "terminal",
        &lower,
        &["terminal", "tty", "shell", "completion"],
        2,
    );
    boost_signal_from_markers(
        keyword_counts,
        "agents",
        &lower,
        &["agent-first", "agent surface", "agent surfaces", "ai coding skills", "ai agents"],
        2,
    );
}

fn tokenize(text: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = STOP_WORDS.iter().copied().collect();
    let mut tokens = HashSet::new();
    let mut current = String::new();

    for character in text.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '+' {
            current.push(character.to_ascii_lowercase());
        } else if !current.is_empty() {
            if !stop_words.contains(current.as_str()) && should_keep_query_token(&current) {
                tokens.insert(current.clone());
            }
            current.clear();
        }
    }

    if !current.is_empty() && !stop_words.contains(current.as_str()) && should_keep_query_token(&current) {
        tokens.insert(current);
    }

    tokens.into_iter().collect()
}

fn collect_match_terms(text: &str) -> HashSet<String> {
    let mut terms = HashSet::new();

    for token in tokenize(text) {
        if should_keep_query_token(&token) {
            terms.insert(token.clone());
        }

        if token.contains('-') {
            for part in token.split('-') {
                if should_keep_query_token(part) {
                    terms.insert(part.to_string());
                }
            }
        }
    }

    terms
}

fn is_controlled_short_token(token: &str) -> bool {
    SHORT_TOKENS.iter().any(|candidate| candidate == &token)
}

fn should_keep_query_token(token: &str) -> bool {
    token.len() >= 3 || is_controlled_short_token(token)
}

fn should_keep_path_token(token: &str) -> bool {
    (token.len() >= 4 || matches!(token, "api" | "cli" | "ops" | "qa" | "ui" | "ux"))
        && !PATH_TOKEN_STOP_WORDS.iter().any(|candidate| candidate == &token)
}

fn classify_signal_kind(token: &str) -> &'static str {
    if matches!(
        token,
        "c" | "c++" | "cpp" | "go" | "javascript" | "js" | "python" | "rust" | "typescript" | "zig"
    ) {
        "language"
    } else if matches!(
        token,
        "bazel" | "cargo" | "clap" | "cmake" | "docker" | "flake" | "gradle" | "jest" | "kubernetes"
            | "nix" | "npm" | "pip" | "pnpm" | "pytest" | "terraform" | "yarn"
    ) {
        "tool"
    } else if matches!(
        token,
        "agent" | "agents" | "api" | "architecture" | "automation" | "backend" | "cli" | "concurrency"
            | "database" | "databases" | "deployment" | "design" | "distributed" | "docs" | "documentation"
            | "frontend" | "networking" | "performance" | "reliability" | "search" | "security" | "systems"
            | "terminal" | "testing" | "ui" | "ux"
    ) {
        "topic"
    } else {
        "keyword"
    }
}

fn insert_signal(
    signal_map: &mut HashMap<(String, String), i64>,
    kind: &'static str,
    value: impl Into<String>,
    weight: i64,
) {
    let value = value.into();
    let entry = signal_map
        .entry((kind.to_string(), value.clone()))
        .or_insert(weight);
    *entry = (*entry).max(weight);
}

fn build_text_signals(text: &str) -> Vec<RecommendSignal> {
    let mut signal_map: HashMap<(String, String), i64> = HashMap::new();

    for token in tokenize(text) {
        if !should_keep_query_token(&token) {
            continue;
        }

        let kind = classify_signal_kind(&token);
        let weight = match kind {
            "language" => 20,
            "tool" => 18,
            "topic" => 16,
            _ => 12,
        };
        insert_signal(&mut signal_map, kind, token, weight);
    }

    let mut signals: Vec<RecommendSignal> = signal_map
        .into_iter()
        .map(|((kind, value), weight)| RecommendSignal {
            kind: match kind.as_str() {
                "language" => "language",
                "tool" => "tool",
                "topic" => "topic",
                _ => "keyword",
            },
            value,
            weight,
        })
        .collect();
    signals.sort_by(|left, right| {
        right
            .weight
            .cmp(&left.weight)
            .then_with(|| left.value.cmp(&right.value))
    });
    signals
}

fn build_project_signals(
    frameworks: &[String],
    config_files: &[String],
    keyword_counts: &HashMap<String, usize>,
    extension_counts: &HashMap<String, usize>,
) -> Vec<RecommendSignal> {
    let mut signal_map: HashMap<(String, String), i64> = HashMap::new();

    for framework in frameworks {
        let extension_hits = match framework.as_str() {
            "rust" => extension_counts.get("rs").copied().unwrap_or(0),
            "python" => extension_counts.get("py").copied().unwrap_or(0),
            "go" => extension_counts.get("go").copied().unwrap_or(0),
            "zig" => extension_counts.get("zig").copied().unwrap_or(0),
            "javascript" => {
                extension_counts.get("js").copied().unwrap_or(0)
                    + extension_counts.get("jsx").copied().unwrap_or(0)
            }
            "typescript" => {
                extension_counts.get("ts").copied().unwrap_or(0)
                    + extension_counts.get("tsx").copied().unwrap_or(0)
            }
            "c" => {
                extension_counts.get("c").copied().unwrap_or(0)
                    + extension_counts.get("h").copied().unwrap_or(0)
            }
            "cpp" => {
                extension_counts.get("cc").copied().unwrap_or(0)
                    + extension_counts.get("cpp").copied().unwrap_or(0)
                    + extension_counts.get("cxx").copied().unwrap_or(0)
                    + extension_counts.get("hpp").copied().unwrap_or(0)
            }
            _ => 0,
        };

        if extension_hits > 0 {
            let weight = 8 + (extension_hits.min(3) as i64) * 4;
            insert_signal(&mut signal_map, "language", framework.clone(), weight);
        }
    }

    for config in config_files {
        match config.to_ascii_lowercase().as_str() {
            "cargo.toml" => {
                insert_signal(&mut signal_map, "language", "rust", 22);
                insert_signal(&mut signal_map, "tool", "cargo", 18);
            }
            "package.json" => {
                insert_signal(&mut signal_map, "language", "javascript", 16);
                insert_signal(&mut signal_map, "language", "typescript", 16);
                insert_signal(&mut signal_map, "tool", "npm", 18);
            }
            "go.mod" => {
                insert_signal(&mut signal_map, "language", "go", 22);
                insert_signal(&mut signal_map, "tool", "go", 18);
            }
            "flake.nix" | "shell.nix" => insert_signal(&mut signal_map, "tool", "nix", 18),
            "dockerfile" | "compose.yaml" | "compose.yml" => {
                insert_signal(&mut signal_map, "tool", "docker", 18);
                insert_signal(&mut signal_map, "topic", "deployment", 16);
            }
            _ => {}
        }
    }

    for (token, count) in keyword_counts {
        let kind = classify_signal_kind(token);
        let base = match kind {
            "tool" => 14,
            "topic" => 14,
            _ => continue,
        };
        let weight = base + (count.saturating_sub(1).min(2) as i64) * 2;
        insert_signal(&mut signal_map, kind, token.clone(), weight);
    }

    let mut signals: Vec<RecommendSignal> = signal_map
        .into_iter()
        .map(|((kind, value), weight)| RecommendSignal {
            kind: match kind.as_str() {
                "language" => "language",
                "tool" => "tool",
                "topic" => "topic",
                _ => "keyword",
            },
            value,
            weight,
        })
        .collect();
    signals.sort_by(|left, right| {
        right
            .weight
            .cmp(&left.weight)
            .then_with(|| left.value.cmp(&right.value))
    });
    signals
}

impl SkillProfile {
    fn from_skill(skill: &Skill) -> Self {
        let mut tag_terms = HashSet::new();
        for tag in &skill.tags {
            tag_terms.extend(collect_match_terms(tag));
        }

        let mut scope_terms = collect_match_terms(&skill.category);
        if let Some(subcategory) = &skill.subcategory {
            scope_terms.extend(collect_match_terms(subcategory));
        }

        let name_terms = collect_match_terms(&skill.name);
        let id_terms = collect_match_terms(&skill.id);
        let description_terms = collect_match_terms(&skill.description);

        Self {
            tag_terms,
            scope_terms,
            name_terms,
            id_terms,
            description_terms,
        }
    }
}

fn match_skill_signal(profile: &SkillProfile, signal: &RecommendSignal) -> Option<ScoreComponent> {
    let value = signal.value.clone();

    if profile.tag_terms.contains(&value) {
        return Some(ScoreComponent {
            feature: "tag".to_string(),
            signal: value,
            weight: signal.weight + 10,
        });
    }

    if profile.scope_terms.contains(&value) {
        return Some(ScoreComponent {
            feature: "scope".to_string(),
            signal: value,
            weight: signal.weight + 8,
        });
    }

    if profile.name_terms.contains(&value) {
        return Some(ScoreComponent {
            feature: "name".to_string(),
            signal: value,
            weight: signal.weight + 6,
        });
    }

    if profile.id_terms.contains(&value) {
        return Some(ScoreComponent {
            feature: "id".to_string(),
            signal: value,
            weight: signal.weight + 5,
        });
    }

    if profile.description_terms.contains(&value) {
        return Some(ScoreComponent {
            feature: "description".to_string(),
            signal: value,
            weight: signal.weight + 3,
        });
    }

    None
}

fn coverage_bonus(
    matched_values: &HashSet<String>,
    matched_kinds: &HashSet<String>,
) -> Vec<ScoreComponent> {
    let mut bonuses = Vec::new();

    if matched_values.len() > 1 {
        bonuses.push(ScoreComponent {
            feature: "coverage".to_string(),
            signal: format!("{} matched signals", matched_values.len()),
            weight: ((matched_values.len() - 1) as i64 * 6).min(18),
        });
    }

    if matched_kinds.contains("language")
        && (matched_kinds.contains("topic") || matched_kinds.contains("tool") || matched_kinds.contains("keyword"))
    {
        bonuses.push(ScoreComponent {
            feature: "coverage".to_string(),
            signal: "mixed intent".to_string(),
            weight: 6,
        });
    }

    bonuses
}

fn reason_from_component(component: &ScoreComponent) -> String {
    match component.feature.as_str() {
        "tag" => format!("matched tag '{}'", component.signal),
        "scope" => format!("matched scope '{}'", component.signal),
        "name" => format!("matched name token '{}'", component.signal),
        "id" => format!("matched id token '{}'", component.signal),
        "description" => format!("matched description token '{}'", component.signal),
        "coverage" => component.signal.clone(),
        _ => format!("matched '{}'", component.signal),
    }
}

fn recommend_skills(manifest: &Manifest, signals: &[RecommendSignal], limit: usize) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();

    for skill in &manifest.skills {
        let profile = SkillProfile::from_skill(skill);
        let mut components = Vec::new();
        let mut matched_values = HashSet::new();
        let mut matched_kinds = HashSet::new();
        let mut strong_matches = 0usize;

        for signal in signals {
            if let Some(component) = match_skill_signal(&profile, signal) {
                if matches!(component.feature.as_str(), "tag" | "scope" | "name" | "id") {
                    strong_matches += 1;
                }
                matched_values.insert(signal.value.clone());
                matched_kinds.insert(signal.kind.to_string());
                components.push(component);
            }
        }

        if matched_values.is_empty() {
            continue;
        }

        components.extend(coverage_bonus(&matched_values, &matched_kinds));
        components.sort_by(|left, right| {
            right
                .weight
                .cmp(&left.weight)
                .then_with(|| left.feature.cmp(&right.feature))
                .then_with(|| left.signal.cmp(&right.signal))
        });

        let score = components.iter().map(|component| component.weight).sum();
        let mut reasons = Vec::new();
        for component in &components {
            let reason = reason_from_component(component);
            if !reasons.contains(&reason) {
                reasons.push(reason);
            }
        }

        recommendations.push(Recommendation {
            skill: skill.clone(),
            score,
            reasons,
            score_breakdown: ScoreBreakdown {
                total: score,
                components,
            },
            matched_signals: matched_values.len(),
            strong_matches,
        });
    }

    recommendations.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.matched_signals.cmp(&left.matched_signals))
            .then_with(|| right.strong_matches.cmp(&left.strong_matches))
            .then_with(|| left.skill.id.cmp(&right.skill.id))
    });
    recommendations.truncate(limit);
    recommendations
}

fn recommendation_json(recommendation: &Recommendation, installed_paths: Vec<String>) -> Value {
    json!({
        "score": recommendation.score,
        "reasons": recommendation.reasons,
        "score_breakdown": recommendation.score_breakdown,
        "skill": skill_summary_json(&recommendation.skill, installed_paths),
    })
}

fn render_catalog_list_text(skills: &[&Skill]) -> String {
    if skills.is_empty() {
        return "No skills matched the requested filters.".to_string();
    }

    let mut grouped: BTreeMap<String, Vec<&Skill>> = BTreeMap::new();
    for skill in skills {
        grouped
            .entry(skill.category.clone())
            .or_default()
            .push(*skill);
    }

    let mut sections = Vec::new();
    for (category, category_skills) in grouped {
        sections.push(format!(
            "{}\n{}",
            category.to_uppercase().bold().cyan(),
            "─".repeat(40).dimmed()
        ));
        for skill in category_skills {
            let subcategory = skill
                .subcategory
                .as_ref()
                .map(|value| format!("[{value}] "))
                .unwrap_or_default();
            sections.push(format!(
                "  {} {}{}",
                skill.id.bold().green(),
                subcategory.dimmed(),
                truncate(&skill.description, 70).dimmed()
            ));
        }
        sections.push(String::new());
    }

    sections.join("\n")
}

fn render_catalog_search_text(results: &[Recommendation]) -> String {
    if results.is_empty() {
        return "No catalog entries matched the query.".to_string();
    }

    let mut lines = vec![
        format!("{} result(s)", results.len()).bold().to_string(),
        String::new(),
    ];
    for recommendation in results {
        lines.push(format!(
            "  {} {}",
            recommendation.skill.id.bold().green(),
            format!("({})", recommendation.score).dimmed()
        ));
        lines.push(format!(
            "    {}",
            truncate(&recommendation.skill.description, 80).dimmed()
        ));
    }
    lines.join("\n")
}

fn render_catalog_show_text(
    skill: &Skill,
    manifest: &Manifest,
    installed_paths: &[String],
) -> String {
    let mut lines = vec![
        skill.id.bold().cyan().to_string(),
        format!("Category: {}", skill.category),
    ];
    if let Some(subcategory) = &skill.subcategory {
        lines.push(format!("Subcategory: {subcategory}"));
    }
    lines.push(String::new());
    lines.push(skill.description.clone());
    lines.push(String::new());
    lines.push(format!("Tags: {}", skill.tags.join(", ")));
    lines.push(format!("Files: {}", skill.files.join(", ")));
    lines.push(format!(
        "Repository: {}/{}",
        manifest.repository, skill.path
    ));
    if installed_paths.is_empty() {
        lines.push("Installed: no".to_string());
    } else {
        lines.push("Installed: yes".to_string());
        for path in installed_paths {
            lines.push(format!("  {}", path.dimmed()));
        }
    }
    lines.join("\n")
}

fn render_install_plan_text(plan: &InstallPlanData, dry_run: bool) -> String {
    let header = if dry_run {
        "Install dry run"
    } else {
        "Install plan"
    };
    let mut lines = vec![
        header.bold().cyan().to_string(),
        format!("Skill: {}", plan.skill.id.green()),
        format!("Scope: {}", plan.scope.as_str()),
        format!("Target: {}", plan.target_dir.display()),
        String::new(),
        "Files:".bold().to_string(),
    ];

    for file in &plan.file_plans {
        lines.push(format!(
            "  {} {} -> {}",
            file.status.to_ascii_uppercase().yellow(),
            file.file,
            file.destination.dimmed()
        ));
    }

    lines.join("\n")
}

fn render_remove_plan_text(plan: &RemovePlanData, dry_run: bool) -> String {
    let header = if dry_run {
        "Remove dry run"
    } else {
        "Remove plan"
    };
    [
        header.bold().cyan().to_string(),
        format!("Skill: {}", plan.request.skill.green()),
        format!("Scope: {}", plan.scope.as_str()),
        format!("Target: {}", plan.target_dir.display()),
        format!("Action: {}", plan.status.to_ascii_uppercase().yellow()),
    ]
    .join("\n")
}

fn render_recommendations_text(
    title: &str,
    input_summary: &str,
    recommendations: &[Recommendation],
    analysis: Option<&ProjectAnalysis>,
) -> String {
    let mut lines = vec![title.bold().cyan().to_string(), input_summary.to_string()];
    if let Some(analysis) = analysis {
        lines.push(String::new());
        lines.push(format!("Files scanned: {}", analysis.total_files));
        if !analysis.frameworks.is_empty() {
            lines.push(format!("Frameworks: {}", analysis.frameworks.join(", ")));
        }
        if !analysis.config_files.is_empty() {
            lines.push(format!(
                "Config files: {}",
                analysis.config_files.join(", ")
            ));
        }
    }

    lines.push(String::new());
    if recommendations.is_empty() {
        lines.push("No recommendations matched the input.".to_string());
        return lines.join("\n");
    }

    for recommendation in recommendations {
        lines.push(format!(
            "  {} {}",
            recommendation.skill.id.bold().green(),
            format!("({})", recommendation.score).dimmed()
        ));
        lines.push(format!("    {}", recommendation.skill.description.dimmed()));
        if !recommendation.reasons.is_empty() {
            lines.push(format!(
                "    {}",
                recommendation.reasons.join("; ").dimmed()
            ));
        }
    }

    lines.join("\n")
}

fn render_env_where_text(paths: &EnvironmentPaths) -> String {
    let mut lines = vec!["Skill install locations".bold().cyan().to_string()];
    if let Some(project_path) = &paths.project_skills_dir {
        let exists = project_path.exists();
        let detail = if exists {
            format!("({} installed)", count_installed_skills(project_path))
        } else {
            "(will be created on demand)".to_string()
        };
        lines.push(format!(
            "  {} {} {}",
            "Project".bold(),
            project_path.display().to_string().green(),
            detail.dimmed()
        ));
    } else {
        lines.push("  Project: none detected".dimmed().to_string());
    }

    let global_detail = if paths.global_skills_dir.exists() {
        format!(
            "({} installed)",
            count_installed_skills(&paths.global_skills_dir)
        )
    } else {
        "(will be created on demand)".to_string()
    };
    lines.push(format!(
        "  {} {} {}",
        "Global".bold(),
        paths.global_skills_dir.display().to_string().green(),
        global_detail.dimmed()
    ));
    lines.join("\n")
}

fn render_doctor_text(checks: &[DoctorCheck]) -> String {
    let mut lines = vec!["sk1llz env doctor".bold().cyan().to_string(), String::new()];
    for check in checks {
        let label = match check.status.as_str() {
            "ok" => "OK".green().bold(),
            "warn" => "WARN".yellow().bold(),
            _ => "ERROR".red().bold(),
        };
        lines.push(format!("  {:<5} {} — {}", label, check.name, check.detail));
        if let Some(fix) = &check.fix {
            lines.push(format!("        {}", format!("Fix: {fix}").dimmed()));
        }
    }
    lines.join("\n")
}

fn render_schema_text(schema: &Value, path: &[String]) -> String {
    let mut lines = Vec::new();
    if !path.is_empty() {
        lines.push(format!("Schema: {}", path.join(" ").bold().cyan()));
    } else {
        lines.push("Schema: sk1llz".bold().cyan().to_string());
    }

    if let Some(about) = schema.get("about").and_then(Value::as_str) {
        lines.push(about.to_string());
    }

    if let Some(usage) = schema.get("usage").and_then(Value::as_str) {
        lines.push(String::new());
        lines.push(format!("Usage: {usage}"));
    }

    if let Some(flags) = schema.get("flags").and_then(Value::as_array) {
        if !flags.is_empty() {
            lines.push(String::new());
            lines.push("Flags:".bold().to_string());
            for flag in flags {
                if let Some(name) = flag.get("name").and_then(Value::as_str) {
                    let description = flag
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    lines.push(format!("  {name} {}", description.dimmed()));
                }
            }
        }
    }

    if let Some(commands) = schema.get("commands").and_then(Value::as_object) {
        if !commands.is_empty() {
            lines.push(String::new());
            lines.push("Subcommands:".bold().to_string());
            for (name, command) in commands {
                let description = command.get("about").and_then(Value::as_str).unwrap_or("");
                lines.push(format!("  {name} {}", description.dimmed()));
            }
        }
    }

    lines.join("\n")
}

fn catalog_filter<'a>(
    skills: &'a [Skill],
    category: &Option<String>,
    tag: &Option<String>,
) -> Vec<&'a Skill> {
    skills
        .iter()
        .filter(|skill| {
            let category_match = category
                .as_ref()
                .map(|value| skill.category.eq_ignore_ascii_case(value))
                .unwrap_or(true);
            let tag_match = tag
                .as_ref()
                .map(|value| skill.tags.iter().any(|tag| tag.eq_ignore_ascii_case(value)))
                .unwrap_or(true);
            category_match && tag_match
        })
        .collect()
}

fn cmd_catalog_list(ctx: &AppContext, args: CatalogListArgs) -> AppResult<Outcome> {
    validate_limit(args.limit)?;
    let fields = parse_fields(&args.fields)?;
    let paths = environment_paths()?;
    let manifest = load_manifest(ctx)?;

    let mut skills = catalog_filter(&manifest.skills, &args.category, &args.tag);
    skills.sort_by(|left, right| left.id.cmp(&right.id));
    let total = skills.len();
    let truncated = total > args.limit;
    skills.truncate(args.limit);

    let items: Vec<Value> = skills
        .iter()
        .map(|skill| skill_summary_json(skill, installed_paths_for_skill(&paths, &skill.id)))
        .collect();

    let value = if let Some(fields) = &fields {
        json!({
            "total": total,
            "count": items.len(),
            "limit": args.limit,
            "items": items.iter().map(|item| filter_object_fields(item, fields)).collect::<Vec<_>>(),
        })
    } else {
        json!({
            "total": total,
            "count": items.len(),
            "limit": args.limit,
            "items": items,
        })
    };

    let mut text = if fields.is_some() {
        render_filtered_text(&value["items"])
    } else {
        render_catalog_list_text(&skills)
    };
    if truncated {
        text.push_str(&format!(
            "\nShowing {} of {} results. Use --limit to see more.",
            items.len(),
            total
        ));
    }

    Ok(Outcome::ok(value, text))
}

fn cmd_catalog_search(ctx: &AppContext, args: CatalogSearchArgs) -> AppResult<Outcome> {
    validate_limit(args.limit)?;
    let fields = parse_fields(&args.fields)?;
    let paths = environment_paths()?;
    let manifest = load_manifest(ctx)?;
    let matcher = SkimMatcherV2::default();

    let mut results = Vec::new();
    for skill in catalog_filter(&manifest.skills, &args.category, &args.tag) {
        let score = matcher.fuzzy_match(&skill.id, &args.query).unwrap_or(0) * 3
            + matcher.fuzzy_match(&skill.name, &args.query).unwrap_or(0) * 2
            + matcher
                .fuzzy_match(&skill.description, &args.query)
                .unwrap_or(0)
            + skill
                .tags
                .iter()
                .filter_map(|tag| matcher.fuzzy_match(tag, &args.query))
                .max()
                .unwrap_or(0);

        if score > 0 {
            results.push(Recommendation {
                skill: skill.clone(),
                score,
                reasons: vec![format!("matched query '{}'", args.query)],
                score_breakdown: ScoreBreakdown {
                    total: score,
                    components: vec![ScoreComponent {
                        feature: "fuzzy".to_string(),
                        signal: args.query.clone(),
                        weight: score,
                    }],
                },
                matched_signals: 1,
                strong_matches: 0,
            });
        }
    }

    results.sort_by(|left, right| right.score.cmp(&left.score));
    results.truncate(args.limit);

    let items: Vec<Value> = results
        .iter()
        .map(|recommendation| {
            recommendation_json(
                recommendation,
                installed_paths_for_skill(&paths, &recommendation.skill.id),
            )
        })
        .collect();

    let value = if let Some(fields) = &fields {
        json!({
            "query": args.query,
            "count": items.len(),
            "items": items.iter().map(|item| filter_recommendation_fields(item, fields)).collect::<Vec<_>>(),
        })
    } else {
        json!({
            "query": args.query,
            "count": items.len(),
            "items": items,
        })
    };

    let text = if fields.is_some() {
        render_filtered_text(&value["items"])
    } else {
        render_catalog_search_text(&results)
    };

    Ok(Outcome::ok(value, text))
}

fn cmd_catalog_show(ctx: &AppContext, args: CatalogShowArgs) -> AppResult<Outcome> {
    let fields = parse_fields(&args.fields)?;
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let skill_ref = validate_skill_ref(&args.skill)?;
    let skill = find_skill(&manifest, &skill_ref)?;
    let item = skill_detail_json(
        skill,
        &manifest,
        installed_paths_for_skill(&paths, &skill.id),
    );

    let value = if let Some(fields) = &fields {
        json!({ "item": filter_object_fields(&item, fields) })
    } else {
        json!({ "item": item.clone() })
    };

    let text = if fields.is_some() {
        render_filtered_text(&value["item"])
    } else {
        render_catalog_show_text(
            skill,
            &manifest,
            &installed_paths_for_skill(&paths, &skill.id),
        )
    };

    Ok(Outcome::ok(value, text))
}

fn cmd_catalog_refresh(ctx: &AppContext, args: RefreshArgs) -> AppResult<Outcome> {
    let manifest = fetch_manifest(ctx, args.dry_run)?;
    let cache_path = manifest_cache_path()?;
    let value = json!({
        "updated": !args.dry_run,
        "dry_run": args.dry_run,
        "cache_path": cache_path,
        "skill_count": manifest.skill_count,
    });
    let text = if args.dry_run {
        format!(
            "Refresh dry run\nCache path: {}\nRemote skills: {}",
            cache_path.display(),
            manifest.skill_count
        )
    } else {
        format!(
            "{} Refreshed the catalog cache at {}\nSkills: {}",
            "✓".green().bold(),
            cache_path.display(),
            manifest.skill_count
        )
    };
    Ok(Outcome::ok(value, text))
}

fn cmd_install_plan(ctx: &AppContext, args: InstallPlanArgs) -> AppResult<Outcome> {
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let plan = build_install_plan(&manifest, resolve_install_request(&args.target)?, &paths)?;
    let value = json!({
        "action": "install",
        "mode": "plan",
        "request": plan.request,
        "scope": plan.scope,
        "target_dir": plan.target_dir,
        "files": plan.file_plans,
    });
    let text = render_install_plan_text(&plan, false);
    Ok(Outcome::ok(value, text))
}

fn cmd_install_apply(ctx: &AppContext, args: InstallApplyArgs) -> AppResult<Outcome> {
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let plan = build_install_plan(&manifest, resolve_install_request(&args.target)?, &paths)?;

    if args.dry_run {
        let value = json!({
            "action": "install",
            "mode": "dry-run",
            "request": plan.request,
            "scope": plan.scope,
            "target_dir": plan.target_dir,
            "files": plan.file_plans,
        });
        let text = render_install_plan_text(&plan, true);
        return Ok(Outcome::ok(value, text));
    }

    confirm_apply(
        ctx,
        &format!(
            "install '{}' into {}",
            plan.skill.id,
            plan.target_dir.display()
        ),
        args.yes,
    )?;
    apply_install_plan(ctx, &plan)?;

    let value = json!({
        "action": "install",
        "mode": "apply",
        "request": plan.request,
        "scope": plan.scope,
        "target_dir": plan.target_dir,
        "files_written": plan.file_plans.len(),
    });
    let text = format!(
        "{} Installed {} into {}",
        "✓".green().bold(),
        plan.skill.id.cyan(),
        plan.target_dir.display()
    );
    Ok(Outcome::ok(value, text))
}

fn cmd_remove_plan(ctx: &AppContext, args: RemovePlanArgs) -> AppResult<Outcome> {
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let plan = build_remove_plan(&manifest, resolve_remove_request(&args.target)?, &paths)?;
    let value = json!({
        "action": "remove",
        "mode": "plan",
        "request": plan.request,
        "scope": plan.scope,
        "target_dir": plan.target_dir,
        "status": plan.status,
    });
    let text = render_remove_plan_text(&plan, false);
    Ok(Outcome::ok(value, text))
}

fn cmd_remove_apply(ctx: &AppContext, args: RemoveApplyArgs) -> AppResult<Outcome> {
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let plan = build_remove_plan(&manifest, resolve_remove_request(&args.target)?, &paths)?;

    if args.dry_run {
        let value = json!({
            "action": "remove",
            "mode": "dry-run",
            "request": plan.request,
            "scope": plan.scope,
            "target_dir": plan.target_dir,
            "status": plan.status,
        });
        let text = render_remove_plan_text(&plan, true);
        return Ok(Outcome::ok(value, text));
    }

    confirm_apply(
        ctx,
        &format!(
            "remove '{}' from {}",
            plan.request.skill,
            plan.target_dir.display()
        ),
        args.yes,
    )?;
    apply_remove_plan(&plan)?;

    let value = json!({
        "action": "remove",
        "mode": "apply",
        "request": plan.request,
        "scope": plan.scope,
        "target_dir": plan.target_dir,
        "removed": true,
    });
    let text = format!(
        "{} Removed {} from {}",
        "✓".green().bold(),
        plan.request.skill.cyan(),
        plan.target_dir.display()
    );
    Ok(Outcome::ok(value, text))
}

fn resolve_recommend_text_input(args: &RecommendTextArgs) -> AppResult<String> {
    if args.stdin {
        return read_stdin_text(
            "pipe a description into stdin or pass it as a positional argument",
        );
    }

    if let Some(description) = &args.description {
        return Ok(description.clone());
    }

    if !io::stdin().is_terminal() {
        return read_stdin_text(
            "pipe a description into stdin or pass it as a positional argument",
        );
    }

    Err(CliError::usage(
        "recommend from-text needs a description",
        "pass a quoted description or re-run with --stdin",
    ))
}

fn cmd_recommend_from_text(ctx: &AppContext, args: RecommendTextArgs) -> AppResult<Outcome> {
    validate_limit(args.read.limit)?;
    let fields = parse_fields(&args.read.fields)?;
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let description = resolve_recommend_text_input(&args)?;
    let signals = build_text_signals(&description);
    let mut tokens: Vec<String> = signals.iter().map(|signal| signal.value.clone()).collect();
    tokens.sort();
    tokens.dedup();
    let recommendations = recommend_skills(&manifest, &signals, args.read.limit);

    let items: Vec<Value> = recommendations
        .iter()
        .map(|recommendation| {
            recommendation_json(
                recommendation,
                installed_paths_for_skill(&paths, &recommendation.skill.id),
            )
        })
        .collect();

    let value = if let Some(fields) = &fields {
        json!({
            "source": "text",
            "input": description,
            "tokens": tokens,
            "count": items.len(),
            "items": items.iter().map(|item| filter_recommendation_fields(item, fields)).collect::<Vec<_>>(),
        })
    } else {
        json!({
            "source": "text",
            "input": description,
            "tokens": tokens,
            "count": items.len(),
            "items": items,
        })
    };

    let text = if fields.is_some() {
        render_filtered_text(&value["items"])
    } else {
        render_recommendations_text(
            "Recommended skills",
            &format!(
                "Input: {}",
                truncate(value["input"].as_str().unwrap_or_default(), 100)
            ),
            &recommendations,
            None,
        )
    };

    Ok(Outcome::ok(value, text))
}

fn cmd_recommend_from_path(ctx: &AppContext, args: RecommendPathArgs) -> AppResult<Outcome> {
    validate_limit(args.read.limit)?;
    let fields = parse_fields(&args.read.fields)?;
    let manifest = load_manifest(ctx)?;
    let paths = environment_paths()?;
    let analysis = analyze_project(&args.path)?;
    let recommendations = recommend_skills(&manifest, &analysis.signals, args.read.limit);

    let items: Vec<Value> = recommendations
        .iter()
        .map(|recommendation| {
            recommendation_json(
                recommendation,
                installed_paths_for_skill(&paths, &recommendation.skill.id),
            )
        })
        .collect();

    let analysis_json = json!({
        "total_files": analysis.total_files,
        "frameworks": analysis.frameworks,
        "config_files": analysis.config_files,
        "extension_counts": analysis.extension_counts,
        "tokens": analysis.tokens,
    });

    let value = if let Some(fields) = &fields {
        json!({
            "source": "path",
            "path": args.path,
            "analysis": analysis_json,
            "count": items.len(),
            "items": items.iter().map(|item| filter_recommendation_fields(item, fields)).collect::<Vec<_>>(),
        })
    } else {
        json!({
            "source": "path",
            "path": args.path,
            "analysis": analysis_json,
            "count": items.len(),
            "items": items,
        })
    };

    let text = if fields.is_some() {
        render_filtered_text(&value["items"])
    } else {
        render_recommendations_text(
            "Recommended skills",
            &format!("Path: {}", args.path.display()),
            &recommendations,
            Some(&analysis),
        )
    };

    Ok(Outcome::ok(value, text))
}

fn cmd_env_where(_ctx: &AppContext) -> AppResult<Outcome> {
    let paths = environment_paths()?;
    let value = json!({
        "active_scope": if paths.project_skills_dir.is_some() { "project" } else { "global" },
        "project": {
            "root": paths.project_root,
            "skills_dir": paths.project_skills_dir,
        },
        "global": {
            "skills_dir": paths.global_skills_dir,
        }
    });
    let text = render_env_where_text(&paths);
    Ok(Outcome::ok(value, text))
}

fn cmd_env_init(_ctx: &AppContext, args: InitArgs) -> AppResult<Outcome> {
    let cwd = std::env::current_dir().map_err(|error| {
        CliError::internal(
            "could not determine the current directory",
            error.to_string(),
        )
    })?;
    let root = find_repo_root(&cwd).unwrap_or(cwd);
    let target = root.join(".claude").join("skills");
    let already_exists = target.exists();

    if !args.dry_run && !already_exists {
        fs::create_dir_all(&target).map_err(|error| {
            CliError::internal(
                format!("failed to create {}", target.display()),
                error.to_string(),
            )
        })?;
        let gitkeep = target.join(".gitkeep");
        if !gitkeep.exists() {
            fs::write(&gitkeep, "").map_err(|error| {
                CliError::internal(
                    format!("failed to write {}", gitkeep.display()),
                    error.to_string(),
                )
            })?;
        }
    }

    let value = json!({
        "path": target,
        "created": !already_exists && !args.dry_run,
        "already_exists": already_exists,
        "dry_run": args.dry_run,
    });
    let text = if already_exists {
        format!(
            "{} Project environment already exists at {}",
            "✓".green().bold(),
            target.display()
        )
    } else if args.dry_run {
        format!("Init dry run\nWould create {}", target.display())
    } else {
        format!(
            "{} Initialized project environment at {}",
            "✓".green().bold(),
            target.display()
        )
    };
    Ok(Outcome::ok(value, text))
}

fn cmd_env_doctor(_ctx: &AppContext) -> AppResult<Outcome> {
    let paths = environment_paths()?;
    let mut checks = Vec::new();

    match cache_dir() {
        Ok(path) => checks.push(DoctorCheck {
            name: "cache directory".to_string(),
            status: "ok".to_string(),
            detail: path.display().to_string(),
            fix: None,
        }),
        Err(error) => checks.push(DoctorCheck {
            name: "cache directory".to_string(),
            status: "error".to_string(),
            detail: error.message,
            fix: error.hint,
        }),
    }

    if let Ok(age) = manifest_age_days() {
        let status = if age < 7 { "ok" } else { "warn" }.to_string();
        let fix = if age < 7 {
            None
        } else {
            Some("run 'sk1llz catalog refresh' to refresh the cache".to_string())
        };
        checks.push(DoctorCheck {
            name: "catalog cache".to_string(),
            status,
            detail: format!("{age} day(s) old"),
            fix,
        });
    } else {
        checks.push(DoctorCheck {
            name: "catalog cache".to_string(),
            status: "warn".to_string(),
            detail: "cache missing".to_string(),
            fix: Some("run 'sk1llz catalog refresh' to create the cache".to_string()),
        });
    }

    let network_status = http_client()?
        .get(manifest_url())
        .send()
        .map(|response| response.status().is_success())
        .unwrap_or(false);
    checks.push(DoctorCheck {
        name: "catalog network".to_string(),
        status: if network_status { "ok" } else { "error" }.to_string(),
        detail: if network_status {
            "remote manifest reachable".to_string()
        } else {
            "failed to reach the remote manifest".to_string()
        },
        fix: if network_status {
            None
        } else {
            Some("check your network connection or SKILLZ_MANIFEST_URL".to_string())
        },
    });

    checks.push(DoctorCheck {
        name: "active scope".to_string(),
        status: "ok".to_string(),
        detail: if paths.project_skills_dir.is_some() {
            "project".to_string()
        } else {
            "global".to_string()
        },
        fix: None,
    });

    let ok = checks.iter().all(|check| check.status == "ok");
    let value = json!({
        "ok": ok,
        "checks": checks,
    });
    let text = render_doctor_text(&checks);
    Ok(Outcome::ok(value, text))
}

fn cli_schema() -> Value {
    json!({
        "name": "sk1llz",
        "about": "Install and recommend AI coding skills",
        "usage": "sk1llz [global flags] <command> [args]",
        "defaults": {
            "format": {
                "tty_stdout": "text",
                "non_tty_stdout": "json"
            }
        },
        "exit_codes": {
            "0": "success",
            "1": "local usage or runtime error",
            "3": "requested item not found",
            "4": "remote network or catalog error"
        },
        "flags": [
            {"name": "--format <text|json>", "description": "Select the output format."},
            {"name": "--json", "description": "Shortcut for --format json."},
            {"name": "--no-color", "description": "Disable color output."},
            {"name": "--quiet", "description": "Reduce non-essential stderr output."},
            {"name": "--verbose", "description": "Increase diagnostic detail on stderr."}
        ],
        "commands": {
            "catalog": {
                "about": "Read and refresh the skill catalog.",
                "usage": "sk1llz catalog <list|search|show|refresh> ...",
                "commands": {
                    "list": {
                        "about": "List skills from the catalog.",
                        "usage": "sk1llz catalog list [--category <category>] [--tag <tag>] [--limit <n>] [--fields <fields>]",
                        "flags": [
                            {"name": "--category <category>", "description": "Filter by top-level category."},
                            {"name": "--tag <tag>", "description": "Filter by tag."},
                            {"name": "--limit <n>", "description": "Limit the number of returned items."},
                            {"name": "--fields <fields>", "description": "Comma-separated item fields to keep."}
                        ]
                    },
                    "search": {
                        "about": "Search skills by name, id, description, or tags.",
                        "usage": "sk1llz catalog search <query> [--category <category>] [--tag <tag>] [--limit <n>] [--fields <fields>]",
                        "flags": [
                            {"name": "--category <category>", "description": "Filter by top-level category."},
                            {"name": "--tag <tag>", "description": "Filter by tag."},
                            {"name": "--limit <n>", "description": "Limit the number of returned items."},
                            {"name": "--fields <fields>", "description": "Comma-separated item fields to keep."}
                        ]
                    },
                    "show": {
                        "about": "Show one skill in detail.",
                        "usage": "sk1llz catalog show <skill> [--fields <fields>]",
                        "flags": [
                            {"name": "--fields <fields>", "description": "Comma-separated item fields to keep."}
                        ]
                    },
                    "refresh": {
                        "about": "Refresh the cached manifest from the remote catalog.",
                        "usage": "sk1llz catalog refresh [--dry-run]",
                        "flags": [
                            {"name": "--dry-run", "description": "Preview the refresh without writing the cache."}
                        ]
                    }
                }
            },
            "install": {
                "about": "Preview or apply skill installation.",
                "usage": "sk1llz install <plan|apply> ...",
                "request_schema": {
                    "skill": "string (required)",
                    "global": "boolean (optional)",
                    "target": "string (optional relative path)"
                },
                "commands": {
                    "plan": {
                        "about": "Preview an install without changing the filesystem.",
                        "usage": "sk1llz install plan <skill> [--global|--target <path>] | --request <json|@file|@->",
                        "flags": [
                            {"name": "--request <json|@file|@->", "description": "Raw JSON request body."},
                            {"name": "--global", "description": "Install into ~/.claude/skills."},
                            {"name": "--target <path>", "description": "Install into a relative path under the current directory."}
                        ]
                    },
                    "apply": {
                        "about": "Apply an install plan.",
                        "usage": "sk1llz install apply <skill> [--global|--target <path>] [--yes] [--dry-run] | --request <json|@file|@->",
                        "flags": [
                            {"name": "--request <json|@file|@->", "description": "Raw JSON request body."},
                            {"name": "--global", "description": "Install into ~/.claude/skills."},
                            {"name": "--target <path>", "description": "Install into a relative path under the current directory."},
                            {"name": "--yes", "description": "Skip the interactive confirmation prompt."},
                            {"name": "--dry-run", "description": "Preview the apply without writing files."}
                        ]
                    }
                }
            },
            "remove": {
                "about": "Preview or apply skill removal.",
                "usage": "sk1llz remove <plan|apply> ...",
                "request_schema": {
                    "skill": "string (required)",
                    "global": "boolean (optional)"
                },
                "commands": {
                    "plan": {
                        "about": "Preview a removal without changing the filesystem.",
                        "usage": "sk1llz remove plan <skill> [--global] | --request <json|@file|@->",
                        "flags": [
                            {"name": "--request <json|@file|@->", "description": "Raw JSON request body."},
                            {"name": "--global", "description": "Remove from ~/.claude/skills."}
                        ]
                    },
                    "apply": {
                        "about": "Apply a removal plan.",
                        "usage": "sk1llz remove apply <skill> [--global] [--yes] [--dry-run] | --request <json|@file|@->",
                        "flags": [
                            {"name": "--request <json|@file|@->", "description": "Raw JSON request body."},
                            {"name": "--global", "description": "Remove from ~/.claude/skills."},
                            {"name": "--yes", "description": "Skip the interactive confirmation prompt."},
                            {"name": "--dry-run", "description": "Preview the apply without deleting files."}
                        ]
                    }
                }
            },
            "recommend": {
                "about": "Recommend skills from text or a project path.",
                "usage": "sk1llz recommend <from-text|from-path> ...",
                "commands": {
                    "from-text": {
                        "about": "Recommend skills from free-form text.",
                        "usage": "sk1llz recommend from-text [description] [--stdin] [--limit <n>] [--fields <fields>]",
                        "flags": [
                            {"name": "--stdin", "description": "Read the description from stdin."},
                            {"name": "--limit <n>", "description": "Limit the number of returned items."},
                            {"name": "--fields <fields>", "description": "Comma-separated item fields to keep."}
                        ]
                    },
                    "from-path": {
                        "about": "Recommend skills from a project path.",
                        "usage": "sk1llz recommend from-path [path] [--limit <n>] [--fields <fields>]",
                        "flags": [
                            {"name": "--limit <n>", "description": "Limit the number of returned items."},
                            {"name": "--fields <fields>", "description": "Comma-separated item fields to keep."}
                        ]
                    }
                }
            },
            "env": {
                "about": "Inspect and prepare the local environment.",
                "usage": "sk1llz env <where|init|doctor>",
                "commands": {
                    "where": {
                        "about": "Show active install locations.",
                        "usage": "sk1llz env where"
                    },
                    "init": {
                        "about": "Initialize a project-local .claude/skills directory.",
                        "usage": "sk1llz env init [--dry-run]",
                        "flags": [
                            {"name": "--dry-run", "description": "Preview the init without creating directories."}
                        ]
                    },
                    "doctor": {
                        "about": "Check cache, network, and active install paths.",
                        "usage": "sk1llz env doctor"
                    }
                }
            },
            "describe": {
                "about": "Print machine-readable command metadata.",
                "usage": "sk1llz describe [command ...]"
            },
            "completions": {
                "about": "Generate shell completion scripts.",
                "usage": "sk1llz completions <shell>",
                "notes": [
                    "Writes shell code to stdout and rejects explicit --json output."
                ]
            }
        }
    })
}

fn select_schema_path(path: &[String]) -> AppResult<Value> {
    let mut node = cli_schema();
    for segment in path {
        let Some(next) = node
            .get("commands")
            .and_then(Value::as_object)
            .and_then(|commands| commands.get(segment))
        else {
            return Err(CliError::not_found(
                format!("command path '{}' is not defined", path.join(" ")),
                "run 'sk1llz describe' to inspect the available command tree",
            ));
        };
        node = next.clone();
    }
    Ok(node)
}

fn cmd_describe(_ctx: &AppContext, args: DescribeArgs) -> AppResult<Outcome> {
    let schema = select_schema_path(&args.path)?;
    let text = render_schema_text(&schema, &args.path);
    Ok(Outcome::ok(schema, text))
}

fn cmd_completions(ctx: &AppContext, args: CompletionArgs) -> AppResult<Outcome> {
    if ctx.explicit_json_requested() {
        return Err(CliError::usage(
            "shell completions do not support JSON output",
            "omit --json/--format json; completions always write shell code to stdout",
        ));
    }

    let mut command = Cli::command();
    generate(args.shell, &mut command, "sk1llz", &mut io::stdout());
    Ok(Outcome::ok(Value::Null, String::new()))
}

fn dispatch(ctx: &AppContext, command: Commands) -> AppResult<Outcome> {
    match command {
        Commands::Catalog { command } => match command {
            CatalogCommand::List(args) => cmd_catalog_list(ctx, args),
            CatalogCommand::Search(args) => cmd_catalog_search(ctx, args),
            CatalogCommand::Show(args) => cmd_catalog_show(ctx, args),
            CatalogCommand::Refresh(args) => cmd_catalog_refresh(ctx, args),
        },
        Commands::Install { command } => match command {
            InstallCommand::Plan(args) => cmd_install_plan(ctx, args),
            InstallCommand::Apply(args) => cmd_install_apply(ctx, args),
        },
        Commands::Remove { command } => match command {
            RemoveCommand::Plan(args) => cmd_remove_plan(ctx, args),
            RemoveCommand::Apply(args) => cmd_remove_apply(ctx, args),
        },
        Commands::Recommend { command } => match command {
            RecommendCommand::FromText(args) => cmd_recommend_from_text(ctx, args),
            RecommendCommand::FromPath(args) => cmd_recommend_from_path(ctx, args),
        },
        Commands::Env { command } => match command {
            EnvCommand::Where => cmd_env_where(ctx),
            EnvCommand::Init(args) => cmd_env_init(ctx, args),
            EnvCommand::Doctor => cmd_env_doctor(ctx),
        },
        Commands::Describe(args) => cmd_describe(ctx, args),
        Commands::Completions(args) => cmd_completions(ctx, args),
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let head: String = text.chars().take(max.saturating_sub(3)).collect();
    format!("{head}...")
}

fn main() {
    let cli = Cli::parse();
    let ctx = AppContext::from_cli(&cli);
    colored::control::set_override(ctx.color_enabled);

    match dispatch(&ctx, cli.command) {
        Ok(outcome) => {
            let exit_code = outcome.exit_code;
            if let Err(error) = emit_outcome(&ctx, outcome) {
                error.emit(&ctx);
                process::exit(error.exit_code);
            }
            if exit_code != 0 {
                process::exit(exit_code);
            }
        }
        Err(error) => {
            let code = error.exit_code;
            error.emit(&ctx);
            process::exit(code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn unique_test_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sk1llz-{name}-{}-{nonce}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_skill(
        id: &str,
        description: &str,
        category: &str,
        subcategory: Option<&str>,
        tags: &[&str],
    ) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            subcategory: subcategory.map(|value| value.to_string()),
            path: format!("{category}/{id}"),
            files: vec!["SKILL.md".to_string()],
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
        }
    }

    fn sample_manifest(skills: Vec<Skill>) -> Manifest {
        Manifest {
            version: "1.0.0".to_string(),
            generated_at: "2026-04-10T00:00:00Z".to_string(),
            repository: "https://example.test/sk1llz".to_string(),
            raw_base_url: "https://example.test/raw".to_string(),
            skill_count: skills.len(),
            skills,
        }
    }

    #[test]
    fn skill_refs_reject_reserved_characters() {
        assert!(validate_skill_ref("hashimoto-cli-ux").is_ok());
        assert!(validate_skill_ref("../escape").is_err());
        assert!(validate_skill_ref("bad?query").is_err());
        assert!(validate_skill_ref("bad%2evalue").is_err());
    }

    #[test]
    fn target_paths_stay_relative() {
        assert!(validate_target_path("./skills").is_ok());
        assert!(validate_target_path("../skills").is_err());
        assert!(validate_target_path("/tmp/skills").is_err());
    }

    #[test]
    fn file_paths_stay_inside_skill_root() {
        assert!(safe_relative_file_path("SKILL.md").is_ok());
        assert!(safe_relative_file_path("references/example.md").is_ok());
        assert!(safe_relative_file_path("../escape").is_err());
    }

    #[test]
    fn tokenize_removes_stop_words() {
        let tokens = tokenize("Build a Rust CLI for distributed systems");
        assert!(tokens.contains(&"rust".to_string()));
        assert!(tokens.contains(&"cli".to_string()));
        assert!(tokens.contains(&"distributed".to_string()));
        assert!(!tokens.contains(&"for".to_string()));
    }

    #[test]
    fn tokenize_keeps_controlled_single_character_tokens() {
        let tokens = tokenize("c programming");
        assert!(tokens.contains(&"c".to_string()));
    }

    #[test]
    fn collect_match_terms_keeps_standalone_c_tag() {
        let terms = collect_match_terms("c");
        assert!(terms.contains("c"));
    }

    #[test]
    fn field_masks_filter_objects() {
        let value = json!({
            "id": "hashimoto-cli-ux",
            "name": "hashimoto",
            "description": "CLI design",
        });
        let filtered = filter_object_fields(&value, &vec!["id".to_string(), "name".to_string()]);
        assert_eq!(
            filtered.get("id").and_then(Value::as_str),
            Some("hashimoto-cli-ux")
        );
        assert_eq!(
            filtered.get("name").and_then(Value::as_str),
            Some("hashimoto")
        );
        assert!(filtered.get("description").is_none());
    }

    #[test]
    fn recommendation_field_masks_project_nested_skill_fields() {
        let value = recommendation_json(
            &Recommendation {
                skill: Skill {
                    id: "hashimoto-cli-ux".to_string(),
                    name: "hashimoto-cli-ux".to_string(),
                    description: "CLI design".to_string(),
                    category: "cli-design".to_string(),
                    subcategory: None,
                    path: "domains/cli-design/hashimoto".to_string(),
                    files: vec!["SKILL.md".to_string()],
                    tags: vec!["cli".to_string()],
                },
                score: 42,
                reasons: vec!["matched query".to_string()],
                score_breakdown: ScoreBreakdown {
                    total: 42,
                    components: vec![ScoreComponent {
                        feature: "tag".to_string(),
                        signal: "cli".to_string(),
                        weight: 42,
                    }],
                },
                matched_signals: 1,
                strong_matches: 1,
            },
            Vec::new(),
        );

        let filtered = filter_recommendation_fields(
            &value,
            &vec!["score".to_string(), "id".to_string(), "name".to_string()],
        );

        assert_eq!(filtered.get("score").and_then(Value::as_i64), Some(42));
        assert_eq!(
            filtered.get("id").and_then(Value::as_str),
            Some("hashimoto-cli-ux")
        );
        assert_eq!(
            filtered.get("name").and_then(Value::as_str),
            Some("hashimoto-cli-ux")
        );
        assert!(filtered.get("skill").is_none());
    }

    #[test]
    fn raw_install_requests_reject_conflicting_scopes() {
        let err = resolve_install_request(&InstallTargetArgs {
            skill: None,
            request: Some(
                r#"{"skill":"hashimoto-cli-ux","global":true,"target":"./custom"}"#.to_string(),
            ),
            global: false,
            target: None,
        })
        .unwrap_err();

        assert_eq!(err.kind, "usage");
        assert!(err
            .message
            .contains("--global and --target cannot be used together"));
    }

    #[test]
    fn project_analysis_respects_depth_budget() {
        let root = unique_test_dir("scan-depth");
        let mut current = root.clone();

        for depth in 0..=PROJECT_SCAN_MAX_DEPTH + 1 {
            fs::write(current.join(format!("depth-{depth}.rs")), "fn main() {}\n").unwrap();
            current = current.join(format!("level-{depth}"));
            fs::create_dir_all(&current).unwrap();
        }

        let analysis = analyze_project(&root).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(analysis.total_files, PROJECT_SCAN_MAX_DEPTH + 1);
        assert!(analysis.frameworks.contains(&"rust".to_string()));
    }

    #[test]
    fn project_analysis_filters_short_extension_tokens() {
        let root = unique_test_dir("scan-tokens");
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(root.join("scripts").join("dev.sh"), "#!/usr/bin/env bash\n").unwrap();
        fs::write(root.join("helper.py"), "print('ok')\n").unwrap();

        let analysis = analyze_project(&root).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(!analysis.tokens.contains(&"rs".to_string()));
        assert!(!analysis.tokens.contains(&"sh".to_string()));
        assert!(!analysis.tokens.contains(&"py".to_string()));
        assert!(analysis.tokens.contains(&"rust".to_string()));
        assert!(analysis.tokens.contains(&"cargo".to_string()));
    }

    #[test]
    fn project_analysis_emits_nix_signal_for_shell_nix() {
        let root = unique_test_dir("scan-shell-nix");
        fs::write(root.join("shell.nix"), "{ pkgs ? import <nixpkgs> {} }: pkgs.mkShell {}\n").unwrap();

        let analysis = analyze_project(&root).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(analysis.config_files.contains(&"shell.nix".to_string()));
        assert!(analysis.tokens.contains(&"nix".to_string()));
    }

    #[test]
    fn project_analysis_ignores_catalog_namespace_tokens() {
        let root = unique_test_dir("scan-catalog");
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
        fs::create_dir_all(root.join("cli").join("src")).unwrap();
        fs::write(root.join("cli").join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::create_dir_all(root.join("languages").join("rust").join("klabnik-teaching-rust")).unwrap();
        fs::write(
            root.join("languages")
                .join("rust")
                .join("klabnik-teaching-rust")
                .join("SKILL.md"),
            "# rust\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("languages").join("python").join("beazley-deep-python")).unwrap();
        fs::write(
            root.join("languages")
                .join("python")
                .join("beazley-deep-python")
                .join("SKILL.md"),
            "# python\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("domains").join("cli-design").join("hashimoto-cli-ux")).unwrap();
        fs::write(
            root.join("domains")
                .join("cli-design")
                .join("hashimoto-cli-ux")
                .join("SKILL.md"),
            "# cli\n",
        )
        .unwrap();

        let analysis = analyze_project(&root).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(analysis.tokens.contains(&"rust".to_string()));
        assert!(analysis.tokens.contains(&"cargo".to_string()));
        assert!(analysis.tokens.contains(&"cli".to_string()));
        assert!(!analysis.tokens.contains(&"klabnik".to_string()));
        assert!(!analysis.tokens.contains(&"beazley".to_string()));
        assert!(!analysis.tokens.contains(&"languages".to_string()));
        assert!(!analysis.tokens.contains(&"python".to_string()));
    }

    #[test]
    fn text_recommendation_prefers_cli_ux_over_generic_rust() {
        let manifest = sample_manifest(vec![
            sample_skill(
                "hashimoto-cli-ux",
                "Design operator-grade CLIs with machine-readable output and terminal UX.",
                "domains",
                Some("cli-design"),
                &["domains", "cli_design", "hashimoto"],
            ),
            sample_skill(
                "bos-concurrency-rust",
                "Design Rust concurrency with atomics and locks.",
                "languages",
                Some("rust"),
                &["languages", "rust", "bos", "concurrency"],
            ),
            sample_skill(
                "klabnik-teaching-rust",
                "Write clear Rust code and documentation for others to learn from.",
                "languages",
                Some("rust"),
                &["languages", "rust", "klabnik", "documentation"],
            ),
        ]);

        let signals = build_text_signals("rust cli ux");
        let recommendations = recommend_skills(&manifest, &signals, 3);

        assert_eq!(recommendations.first().map(|item| item.skill.id.as_str()), Some("hashimoto-cli-ux"));
        assert!(recommendations[0].score > recommendations[1].score);
    }

    #[test]
    fn path_recommendation_does_not_promote_python_for_rust_cli_repo() {
        let root = unique_test_dir("recommend-path");
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::create_dir_all(root.join("cli")).unwrap();
        fs::write(root.join("cli").join("guide.md"), "# cli\n").unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(root.join("scripts").join("helper.py"), "print('ok')\n").unwrap();

        let analysis = analyze_project(&root).unwrap();
        let _ = fs::remove_dir_all(&root);

        let manifest = sample_manifest(vec![
            sample_skill(
                "beazley-deep-python",
                "Protocol-first guidance for advanced Python generators and descriptors.",
                "languages",
                Some("python"),
                &["languages", "python", "beazley", "streaming"],
            ),
            sample_skill(
                "hashimoto-cli-ux",
                "Design operator-grade CLIs with machine-readable output and terminal UX.",
                "domains",
                Some("cli-design"),
                &["domains", "cli_design", "hashimoto"],
            ),
            sample_skill(
                "klabnik-teaching-rust",
                "Write clear Rust code and documentation for others to learn from.",
                "languages",
                Some("rust"),
                &["languages", "rust", "klabnik", "documentation", "cargo"],
            ),
        ]);

        let recommendations = recommend_skills(&manifest, &analysis.signals, 3);
        let ids: Vec<&str> = recommendations.iter().map(|item| item.skill.id.as_str()).collect();

        assert_eq!(ids.first().copied(), Some("klabnik-teaching-rust"));
        assert!(ids.iter().position(|id| *id == "beazley-deep-python").unwrap() > ids.iter().position(|id| *id == "hashimoto-cli-ux").unwrap());
    }

    #[test]
    fn path_recommendation_uses_readme_intent_for_cli_repo() {
        let root = unique_test_dir("recommend-docs");
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            root.join("README.md"),
            "# demo\nA command-line tool for AI agents with machine-readable output.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("cli")).unwrap();
        fs::write(
            root.join("cli").join("README.md"),
            "## CLI\nOutput contract: stdout, stderr, --json, shell completions, terminal UX.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("scripts")).unwrap();
        fs::write(root.join("scripts").join("helper.py"), "print('ok')\n").unwrap();

        let analysis = analyze_project(&root).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert!(analysis.tokens.contains(&"cli".to_string()));
        assert!(analysis.tokens.contains(&"automation".to_string()));
        assert!(analysis.tokens.contains(&"terminal".to_string()));
        assert!(analysis.tokens.contains(&"agents".to_string()));

        let manifest = sample_manifest(vec![
            sample_skill(
                "beazley-deep-python",
                "Protocol-first guidance for advanced Python generators and descriptors.",
                "languages",
                Some("python"),
                &["languages", "python", "beazley", "streaming"],
            ),
            sample_skill(
                "hashimoto-cli-ux",
                "Design operator-grade CLIs with machine-readable output, automation-safe JSON, and terminal UX for AI agents.",
                "domains",
                Some("cli-design"),
                &["domains", "cli_design", "hashimoto"],
            ),
            sample_skill(
                "klabnik-teaching-rust",
                "Write clear Rust code and documentation for others to learn from.",
                "languages",
                Some("rust"),
                &["languages", "rust", "klabnik", "documentation", "cargo"],
            ),
        ]);

        let recommendations = recommend_skills(&manifest, &analysis.signals, 3);
        assert_eq!(
            recommendations.first().map(|item| item.skill.id.as_str()),
            Some("hashimoto-cli-ux")
        );
    }

    #[test]
    fn recommend_from_path_json_smoke_covers_field_mask_and_score_breakdown() {
        let _guard = test_env_lock().lock().unwrap();
        let repo_root = unique_test_dir("recommend-json");
        fs::write(repo_root.join("Cargo.toml"), "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
        fs::create_dir_all(repo_root.join("src")).unwrap();
        fs::write(repo_root.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            repo_root.join("README.md"),
            "# demo\nAn agent-first CLI with machine-readable output.\n",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("cli")).unwrap();
        fs::write(
            repo_root.join("cli").join("README.md"),
            "## CLI\nOutput contract: stdout, stderr, --json, dry-run, terminal UX.\n",
        )
        .unwrap();

        let manifest = sample_manifest(vec![
            sample_skill(
                "hashimoto-cli-ux",
                "Design operator-grade CLIs with machine-readable output, automation-safe JSON, and terminal UX for AI agents.",
                "domains",
                Some("cli-design"),
                &["domains", "cli_design", "hashimoto"],
            ),
            sample_skill(
                "klabnik-teaching-rust",
                "Write clear Rust code and documentation for others to learn from.",
                "languages",
                Some("rust"),
                &["languages", "rust", "klabnik", "documentation", "cargo"],
            ),
        ]);

        let original_dir = std::env::current_dir().unwrap();
        let old_cache = std::env::var_os("XDG_CACHE_HOME");
        let old_home = std::env::var_os("HOME");
        let cache_root = repo_root.join("cache");
        fs::create_dir_all(cache_root.join("sk1llz")).unwrap();
        fs::write(
            cache_root.join("sk1llz").join("skills.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        std::env::set_current_dir(&repo_root).unwrap();
        std::env::set_var("XDG_CACHE_HOME", &cache_root);
        std::env::set_var("HOME", repo_root.join("home"));

        let ctx = AppContext {
            format: OutputFormat::Json,
            quiet: false,
            verbose: 0,
            color_enabled: false,
            explicit_json: true,
        };
        let outcome = cmd_recommend_from_path(
            &ctx,
            RecommendPathArgs {
                path: repo_root.clone(),
                read: RecommendReadArgs {
                    limit: 3,
                    fields: Some("score,reasons,score_breakdown,id,name".to_string()),
                },
            },
        )
        .unwrap();

        std::env::set_current_dir(original_dir).unwrap();
        match old_cache {
            Some(value) => std::env::set_var("XDG_CACHE_HOME", value),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
        match old_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }

        let _ = fs::remove_dir_all(&repo_root);

        let encoded = serde_json::to_string_pretty(&outcome.response.value).unwrap();
        let parsed: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(parsed.get("source").and_then(Value::as_str), Some("path"));

        let first = parsed
            .get("items")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_object)
            .unwrap();
        let keys: HashSet<&str> = first.keys().map(|key| key.as_str()).collect();
        assert_eq!(
            keys,
            HashSet::from(["score", "reasons", "score_breakdown", "id", "name"])
        );
        assert_eq!(first.get("id").and_then(Value::as_str), Some("hashimoto-cli-ux"));
        assert_eq!(
            first.get("score").and_then(Value::as_i64),
            first.get("score_breakdown")
                .and_then(|value| value.get("total"))
                .and_then(Value::as_i64)
        );
    }

    #[test]
    fn completions_reject_explicit_json_output() {
        let err = cmd_completions(
            &AppContext {
                format: OutputFormat::Json,
                quiet: false,
                verbose: 0,
                color_enabled: false,
                explicit_json: true,
            },
            CompletionArgs { shell: Shell::Bash },
        )
        .unwrap_err();

        assert_eq!(err.kind, "usage");
        assert!(err.message.contains("do not support JSON output"));
    }
}
