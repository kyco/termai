# Task: Add Plugin System and Extensibility Framework

## Priority: Low
## Estimated Effort: 7-10 days
## Dependencies: None

## Overview
Implement a plugin system that allows users and developers to extend TermAI functionality through custom plugins. This addresses the need for workflow integration and custom tooling while keeping the core application focused.

## Requirements

### Functional Requirements
1. **Plugin Types**
   - **Command Plugins**: Custom commands triggered by `/command`
   - **Hook Plugins**: React to events (message sent, session created, etc.)
   - **Provider Plugins**: Additional LLM providers
   - **Export Plugins**: Custom export formats
   - **UI Plugins**: Custom widgets and overlays

2. **Plugin Management**
   - Install/uninstall plugins
   - Enable/disable plugins
   - Plugin configuration
   - Version management
   - Dependency resolution

3. **Security**
   - Sandboxed execution
   - Permission system
   - Code signing (optional)
   - Safe plugin API

### Technical Requirements
1. **Plugin Architecture**
   ```rust
   pub trait Plugin: Send + Sync {
       fn name(&self) -> &str;
       fn version(&self) -> &str;
       fn description(&self) -> &str;
       fn initialize(&mut self, context: &PluginContext) -> Result<()>;
       fn shutdown(&mut self) -> Result<()>;
   }
   
   pub trait CommandPlugin: Plugin {
       fn command_name(&self) -> &str;
       fn execute(&self, args: &[String], context: &ExecutionContext) -> Result<CommandResult>;
       fn help(&self) -> String;
   }
   
   pub trait HookPlugin: Plugin {
       fn hooks(&self) -> Vec<HookType>;
       fn on_hook(&self, hook: HookType, data: HookData, context: &HookContext) -> Result<()>;
   }
   
   #[derive(Clone, Debug)]
   pub enum HookType {
       MessageSent,
       MessageReceived,
       SessionCreated,
       SessionDeleted,
       AppStarted,
       AppShutdown,
   }
   ```

2. **Plugin Registry**
   ```rust
   pub struct PluginRegistry {
       plugins: HashMap<String, Box<dyn Plugin>>,
       command_plugins: HashMap<String, Box<dyn CommandPlugin>>,
       hook_plugins: HashMap<HookType, Vec<Box<dyn HookPlugin>>>,
       config: PluginConfig,
   }
   
   impl PluginRegistry {
       pub fn load_plugin(&mut self, path: &Path) -> Result<()>;
       pub fn unload_plugin(&mut self, name: &str) -> Result<()>;
       pub fn execute_command(&self, command: &str, args: &[String]) -> Result<CommandResult>;
       pub fn trigger_hook(&self, hook: HookType, data: HookData) -> Result<()>;
       pub fn list_plugins(&self) -> Vec<PluginInfo>;
   }
   ```

## Implementation Steps

1. **Plugin Framework Foundation**
   ```rust
   // plugin/mod.rs
   pub mod registry;
   pub mod loader;
   pub mod api;
   pub mod sandbox;
   
   use libloading::Library;
   
   pub struct PluginLoader {
       loaded_libraries: HashMap<String, Library>,
   }
   
   impl PluginLoader {
       pub fn load_from_path(&mut self, path: &Path) -> Result<Box<dyn Plugin>> {
           unsafe {
               let lib = Library::new(path)?;
               
               // Look for plugin entry point
               let create_plugin: Symbol<unsafe extern fn() -> *mut dyn Plugin> = 
                   lib.get(b"create_plugin")?;
               
               let plugin = Box::from_raw(create_plugin());
               
               self.loaded_libraries.insert(plugin.name().to_string(), lib);
               Ok(plugin)
           }
       }
   }
   ```

2. **Plugin API Definition**
   ```rust
   // plugin/api.rs
   pub struct PluginContext {
       pub app_version: String,
       pub data_dir: PathBuf,
       pub config_dir: PathBuf,
       pub permissions: PluginPermissions,
   }
   
   pub struct ExecutionContext {
       pub current_session: Option<String>,
       pub user_input: String,
       pub session_service: Arc<dyn SessionServiceTrait>,
       pub message_service: Arc<dyn MessageServiceTrait>,
   }
   
   #[derive(Clone)]
   pub struct PluginPermissions {
       pub can_read_sessions: bool,
       pub can_write_sessions: bool,
       pub can_access_network: bool,
       pub can_access_filesystem: bool,
       pub can_execute_commands: bool,
   }
   
   #[derive(Debug)]
   pub struct CommandResult {
       pub output: String,
       pub success: bool,
       pub data: Option<serde_json::Value>,
   }
   ```

3. **Built-in Example Plugins**
   ```rust
   // plugins/git_plugin.rs
   pub struct GitPlugin {
       name: String,
   }
   
   impl Plugin for GitPlugin {
       fn name(&self) -> &str { "git" }
       fn version(&self) -> &str { "1.0.0" }
       fn description(&self) -> &str { "Git integration plugin" }
       
       fn initialize(&mut self, _context: &PluginContext) -> Result<()> {
           Ok(())
       }
       
       fn shutdown(&mut self) -> Result<()> {
           Ok(())
       }
   }
   
   impl CommandPlugin for GitPlugin {
       fn command_name(&self) -> &str { "git" }
       
       fn execute(&self, args: &[String], context: &ExecutionContext) -> Result<CommandResult> {
           match args.get(0).map(|s| s.as_str()) {
               Some("status") => {
                   let output = std::process::Command::new("git")
                       .arg("status")
                       .arg("--porcelain")
                       .output()?;
                   
                   Ok(CommandResult {
                       output: String::from_utf8_lossy(&output.stdout).to_string(),
                       success: output.status.success(),
                       data: None,
                   })
               }
               Some("diff") => {
                   let output = std::process::Command::new("git")
                       .arg("diff")
                       .output()?;
                   
                   Ok(CommandResult {
                       output: String::from_utf8_lossy(&output.stdout).to_string(),
                       success: output.status.success(),
                       data: None,
                   })
               }
               _ => Ok(CommandResult {
                   output: "Usage: /git status|diff".to_string(),
                   success: false,
                   data: None,
               })
           }
       }
       
       fn help(&self) -> String {
           "Git integration commands:\n/git status - Show git status\n/git diff - Show git diff".to_string()
       }
   }
   ```

4. **Plugin Command Integration**
   ```rust
   // In event handling
   impl EventHandler {
       fn handle_command_input(&mut self, app: &mut App, input: &str) -> Result<()> {
           if let Some(stripped) = input.strip_prefix('/') {
               let parts: Vec<&str> = stripped.split_whitespace().collect();
               if let Some(command) = parts.first() {
                   let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                   
                   match app.plugin_registry.execute_command(command, &args) {
                       Ok(result) => {
                           app.add_system_message(&result.output);
                           if let Some(data) = result.data {
                               // Handle structured data from plugin
                               app.handle_plugin_data(data);
                           }
                       }
                       Err(e) => {
                           app.add_error_message(&format!("Plugin error: {}", e));
                       }
                   }
               }
           }
           Ok(())
       }
   }
   ```

5. **Plugin Management UI**
   ```rust
   pub struct PluginManagerUI {
       plugins: Vec<PluginInfo>,
       selected_index: usize,
       mode: PluginManagerMode,
   }
   
   #[derive(Clone)]
   pub enum PluginManagerMode {
       List,
       Install,
       Configure(String),
   }
   
   fn draw_plugin_manager(f: &mut Frame, manager: &PluginManagerUI, area: Rect) {
       let chunks = Layout::default()
           .direction(Direction::Horizontal)
           .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
           .split(area);
       
       // Plugin list
       let plugin_items: Vec<ListItem> = manager.plugins.iter()
           .map(|plugin| {
               let status = if plugin.enabled { "✓" } else { "○" };
               let item_text = format!("{} {} v{}", status, plugin.name, plugin.version);
               ListItem::new(item_text)
           })
           .collect();
       
       let plugin_list = List::new(plugin_items)
           .block(Block::default().title("Plugins").borders(Borders::ALL))
           .highlight_style(Style::default().bg(Color::Blue));
       
       f.render_stateful_widget(plugin_list, chunks[0], &mut manager.list_state);
       
       // Plugin details
       if let Some(selected_plugin) = manager.plugins.get(manager.selected_index) {
           draw_plugin_details(f, selected_plugin, chunks[1]);
       }
   }
   ```

## Built-in Plugin Examples

1. **Git Integration Plugin**
   - Commands: `/git status`, `/git diff`, `/git log`
   - Hooks: Auto-include git context in relevant conversations

2. **File System Plugin**
   - Commands: `/ls`, `/cat`, `/find`
   - File content inclusion helpers

3. **Docker Plugin**
   - Commands: `/docker ps`, `/docker logs`
   - Container management integration

4. **Jira Plugin**
   - Commands: `/jira create`, `/jira search`
   - Issue tracking integration

5. **Export Plugin**
   - Additional export formats (PDF, HTML, Word)
   - Custom export templates

## Plugin Development Kit

```rust
// Example plugin template
use termai_plugin_api::{Plugin, CommandPlugin, PluginContext, ExecutionContext, CommandResult};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my_plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "My custom plugin" }
    
    fn initialize(&mut self, context: &PluginContext) -> Result<()> {
        // Plugin initialization
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        // Cleanup
        Ok(())
    }
}

impl CommandPlugin for MyPlugin {
    fn command_name(&self) -> &str { "my_command" }
    
    fn execute(&self, args: &[String], context: &ExecutionContext) -> Result<CommandResult> {
        // Command implementation
        Ok(CommandResult {
            output: "Hello from my plugin!".to_string(),
            success: true,
            data: None,
        })
    }
    
    fn help(&self) -> String {
        "My custom command help".to_string()
    }
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin))
}
```

## Testing Requirements
- Unit tests for plugin loading/unloading
- Integration tests for command execution
- Security tests for sandboxing
- Performance tests for plugin overhead
- UI tests for plugin management

## Acceptance Criteria
- [ ] Plugins can be loaded from dynamic libraries
- [ ] Command plugins work correctly
- [ ] Hook system triggers appropriately
- [ ] Plugin management UI is functional
- [ ] Security permissions are enforced
- [ ] Built-in example plugins work
- [ ] Plugin development is well-documented

## Security Considerations
- Plugins run in restricted environment
- File system access limited to plugin directory
- Network access requires explicit permission
- No direct access to application internals
- Code signing for trusted plugins

## Future Enhancements
- Plugin marketplace
- Remote plugin installation
- Plugin auto-updates
- WebAssembly plugin support
- Plugin analytics and monitoring
- Community plugin sharing
- Plugin development IDE integration