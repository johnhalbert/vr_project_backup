// Scripting utilities for CLI
use anyhow::{Result, Context, anyhow};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::collections::HashMap;

/// Script execution context
pub struct ScriptContext {
    /// Variables available to the script
    pub variables: HashMap<String, String>,
    /// Working directory for script execution
    pub working_dir: PathBuf,
    /// Whether to continue execution on error
    pub continue_on_error: bool,
    /// Maximum execution time in seconds (0 = no limit)
    pub timeout: u64,
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            continue_on_error: false,
            timeout: 0,
        }
    }
}

/// Execute a script file
pub fn execute_script(script_path: &Path, context: &mut ScriptContext) -> Result<()> {
    // Check if script exists
    if !script_path.exists() {
        return Err(anyhow!("Script file does not exist: {}", script_path.display()));
    }
    
    // Read script content
    let mut file = File::open(script_path).context(format!("Failed to open script file: {}", script_path.display()))?;
    let mut content = String::new();
    file.read_to_string(&mut content).context(format!("Failed to read script file: {}", script_path.display()))?;
    
    // Execute script
    execute_script_content(&content, context)
}

/// Execute script content
pub fn execute_script_content(content: &str, context: &mut ScriptContext) -> Result<()> {
    // Parse script lines
    let lines: Vec<&str> = content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    
    // Execute each line
    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        
        // Check for variable assignment
        if line.contains('=') && !line.starts_with("if ") && !line.contains(" == ") && !line.contains(" != ") {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let var_value = parts[1].trim();
                
                // Handle quoted values
                let var_value = if (var_value.starts_with('"') && var_value.ends_with('"')) || 
                                  (var_value.starts_with('\'') && var_value.ends_with('\'')) {
                    &var_value[1..var_value.len()-1]
                } else {
                    var_value
                };
                
                // Store variable
                context.variables.insert(var_name.to_string(), var_value.to_string());
                continue;
            }
        }
        
        // Replace variables in command
        let mut command = line.to_string();
        for (var_name, var_value) in &context.variables {
            command = command.replace(&format!("${{{}}}", var_name), var_value);
            command = command.replace(&format!("${}", var_name), var_value);
        }
        
        // Execute command
        match execute_command(&command, &context.working_dir) {
            Ok(output) => {
                println!("{}", output);
            },
            Err(e) => {
                eprintln!("Error at line {}: {}", line_num, e);
                if !context.continue_on_error {
                    return Err(anyhow!("Script execution failed at line {}: {}", line_num, e));
                }
            }
        }
    }
    
    Ok(())
}

/// Execute a shell command
fn execute_command(command: &str, working_dir: &Path) -> Result<String> {
    // Split command into program and arguments
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
    let args: Vec<&str> = parts.collect();
    
    // Execute command
    let output = Command::new(program)
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("Failed to execute command: {}", command))?;
    
    // Check if command succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Command failed: {}", stderr));
    }
    
    // Return stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// Create a new script file
pub fn create_script(script_path: &Path, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent).context(format!("Failed to create directory: {}", parent.display()))?;
    }
    
    // Write script content
    let mut file = File::create(script_path).context(format!("Failed to create script file: {}", script_path.display()))?;
    file.write_all(content.as_bytes()).context(format!("Failed to write to script file: {}", script_path.display()))?;
    
    // Make script executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(script_path)
            .context(format!("Failed to get metadata for script file: {}", script_path.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(script_path, perms)
            .context(format!("Failed to set permissions for script file: {}", script_path.display()))?;
    }
    
    Ok(())
}

/// Get the scripts directory
pub fn get_scripts_dir() -> Result<PathBuf> {
    let scripts_dir = crate::utils::file::get_data_dir()?.join("scripts");
    crate::utils::file::ensure_dir_exists(&scripts_dir)?;
    Ok(scripts_dir)
}

/// List available scripts
pub fn list_scripts() -> Result<Vec<PathBuf>> {
    let scripts_dir = get_scripts_dir()?;
    crate::utils::file::list_files_with_extension(&scripts_dir, "sh")
}
