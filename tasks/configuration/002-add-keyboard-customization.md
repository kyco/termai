# Task: Add Keyboard Shortcut Customization

## Priority: Low
## Estimated Effort: 2-3 days
## Dependencies: None

## Overview
Allow users to customize keyboard shortcuts and keybindings to match their preferences and workflows. This addresses power users who want to optimize their interaction patterns or adapt to different terminal environments.

## Requirements

### Functional Requirements
1. **Keybinding Management**
   - View all current keybindings
   - Modify existing shortcuts
   - Add custom shortcuts for actions
   - Reset to defaults
   - Import/export keybinding profiles

2. **Conflict Detection**
   - Warn about conflicting keybindings
   - Suggest alternatives
   - Show which actions would be affected
   - Allow force override with warning

3. **Keybinding Profiles**
   - Built-in profiles (Default, Vim, Emacs)
   - Custom profile creation
   - Quick profile switching
   - Profile inheritance

### Technical Requirements
1. **Keybinding Configuration**
   ```rust
   #[derive(Serialize, Deserialize, Clone)]
   pub struct KeybindingConfig {
       pub profile_name: String,
       pub bindings: HashMap<String, KeyBinding>,
       pub modifiers: ModifierConfig,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub struct KeyBinding {
       pub action: Action,
       pub key: KeyCode,
       pub modifiers: KeyModifiers,
       pub context: Context,
       pub description: String,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum Action {
       Navigation(NavigationAction),
       Session(SessionAction),
       Input(InputAction),
       Search(SearchAction),
       System(SystemAction),
       Custom(String),
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum Context {
       Global,
       SessionList,
       Chat,
       Input,
       Settings,
       Search,
   }
   ```

2. **Keybinding Service**
   ```rust
   pub struct KeybindingService {
       config: KeybindingConfig,
       profiles: HashMap<String, KeybindingConfig>,
       config_repo: Arc<ConfigRepository>,
   }
   
   impl KeybindingService {
       pub fn get_action_for_key(&self, key: KeyEvent, context: Context) -> Option<Action>;
       pub fn set_binding(&mut self, action: Action, binding: KeyBinding) -> Result<()>;
       pub fn remove_binding(&mut self, action: Action) -> Result<()>;
       pub fn check_conflicts(&self, binding: &KeyBinding) -> Vec<Conflict>;
       pub fn load_profile(&mut self, name: &str) -> Result<()>;
       pub fn save_profile(&self, name: &str) -> Result<()>;
   }
   ```

## Implementation Steps

1. **Default Keybinding Profiles**
   ```rust
   // keybindings/profiles.rs
   impl KeybindingConfig {
       pub fn default_profile() -> Self {
           let mut bindings = HashMap::new();
           
           // Navigation
           bindings.insert("quit".to_string(), KeyBinding {
               action: Action::System(SystemAction::Quit),
               key: KeyCode::Char('q'),
               modifiers: KeyModifiers::ALT,
               context: Context::Global,
               description: "Quit application".to_string(),
           });
           
           bindings.insert("tab_next".to_string(), KeyBinding {
               action: Action::Navigation(NavigationAction::NextPanel),
               key: KeyCode::Tab,
               modifiers: KeyModifiers::NONE,
               context: Context::Global,
               description: "Move to next panel".to_string(),
           });
           
           // Session management
           bindings.insert("new_session".to_string(), KeyBinding {
               action: Action::Session(SessionAction::New),
               key: KeyCode::Char('n'),
               modifiers: KeyModifiers::CONTROL,
               context: Context::Global,
               description: "Create new session".to_string(),
           });
           
           KeybindingConfig {
               profile_name: "Default".to_string(),  
               bindings,
               modifiers: ModifierConfig::default(),
           }
       }
       
       pub fn vim_profile() -> Self {
           let mut config = Self::default_profile();
           config.profile_name = "Vim".to_string();
           
           // Override with vim-style bindings
           config.bindings.insert("move_up".to_string(), KeyBinding {
               action: Action::Navigation(NavigationAction::Up),
               key: KeyCode::Char('k'),
               modifiers: KeyModifiers::NONE,
               context: Context::SessionList,
               description: "Move up (Vim style)".to_string(),
           });
           
           config.bindings.insert("move_down".to_string(), KeyBinding {
               action: Action::Navigation(NavigationAction::Down),
               key: KeyCode::Char('j'),
               modifiers: KeyModifiers::NONE,
               context: Context::SessionList,
               description: "Move down (Vim style)".to_string(),
           });
           
           config
       }
   }
   ```

2. **Keybinding Editor UI**
   ```rust
   pub struct KeybindingEditor {
       config: KeybindingConfig,
       selected_action: Option<String>,
       editing_binding: Option<KeyBinding>,
       conflict_warnings: Vec<Conflict>,
       filter_context: Option<Context>,
   }
   
   impl KeybindingEditor {
       pub fn new(config: KeybindingConfig) -> Self;
       pub fn edit_binding(&mut self, action: &str);
       pub fn save_binding(&mut self, binding: KeyBinding) -> Result<()>;
       pub fn check_for_conflicts(&mut self);
       pub fn reset_to_defaults(&mut self);
   }
   ```

3. **Key Capture Widget**
   ```rust
   pub struct KeyCaptureWidget {
       capturing: bool,
       current_keys: Vec<KeyEvent>,
       display_string: String,
   }
   
   impl KeyCaptureWidget {
       pub fn start_capture(&mut self);
       pub fn handle_key(&mut self, key: KeyEvent) -> Option<KeyBinding>;
       pub fn cancel_capture(&mut self);
   }
   
   // In event handling
   fn draw_key_capture(f: &mut Frame, widget: &KeyCaptureWidget, area: Rect) {
       let text = if widget.capturing {
           format!("Press keys for binding... ({})", widget.display_string)
       } else {
           "Click to set keybinding".to_string()
       };
       
       let paragraph = Paragraph::new(text)
           .style(Style::default().fg(Color::Yellow))
           .block(Block::default().borders(Borders::ALL));
           
       f.render_widget(paragraph, area);
   }
   ```

4. **Dynamic Event Handling**
   ```rust
   // Modify events.rs to use keybinding service
   impl EventHandler {
       pub fn handle_key_event(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
           let context = self.get_current_context(app);
           
           if let Some(action) = app.keybinding_service.get_action_for_key(key, context) {
               self.execute_action(app, action)?;
           } else {
               // Handle as regular key input if no binding found
               self.handle_default_key(app, key)?;
           }
           
           Ok(())
       }
       
       fn execute_action(&mut self, app: &mut App, action: Action) -> Result<()> {
           match action {
               Action::Navigation(nav_action) => self.handle_navigation(app, nav_action),
               Action::Session(session_action) => self.handle_session_action(app, session_action),
               Action::System(system_action) => self.handle_system_action(app, system_action),
               Action::Custom(command) => self.handle_custom_action(app, command),
               _ => Ok(()),
           }
       }
   }
   ```

5. **Keybinding Settings UI**
   ```rust
   fn draw_keybinding_settings(f: &mut Frame, app: &App, area: Rect) {
       let chunks = Layout::default()
           .direction(Direction::Horizontal)
           .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
           .split(area);
       
       // Left side: Action list
       let actions: Vec<ListItem> = app.keybinding_editor.get_actions()
           .iter()
           .map(|(name, binding)| {
               let key_display = format_key_binding(&binding);
               ListItem::new(format!("{}: {}", name, key_display))
           })
           .collect();
       
       let action_list = List::new(actions)
           .block(Block::default().title("Actions").borders(Borders::ALL))
           .highlight_style(Style::default().bg(Color::Blue));
       
       f.render_stateful_widget(action_list, chunks[0], &mut app.action_list_state);
       
       // Right side: Binding editor
       if let Some(selected_action) = app.keybinding_editor.get_selected_action() {
           draw_binding_editor(f, app, chunks[1], selected_action);
       }
   }
   ```

## Testing Requirements
- Unit tests for keybinding resolution
- Conflict detection tests
- Profile loading/saving tests
- UI tests for keybinding editor
- Key capture functionality tests

## Acceptance Criteria
- [ ] Users can view all current keybindings
- [ ] Individual keybindings can be modified
- [ ] Conflict detection works correctly
- [ ] Multiple profiles can be saved and loaded
- [ ] Key capture interface is intuitive
- [ ] Changes apply immediately
- [ ] Export/import functionality works

## Built-in Profiles

### Default Profile
- Standard terminal application shortcuts
- Ctrl+C, Ctrl+N, Tab navigation
- Arrow keys for movement

### Vim Profile  
- hjkl for navigation
- :q to quit
- / for search
- Visual mode with v/V

### Emacs Profile
- Ctrl+X prefixes for commands
- Meta key combinations
- Emacs-style navigation

## Configuration File Format
```json
{
  "profile_name": "Custom",
  "bindings": {
    "quit": {
      "action": {"System": "Quit"},
      "key": {"Char": "q"},
      "modifiers": "ALT",
      "context": "Global",
      "description": "Quit application"
    },
    "new_session": {
      "action": {"Session": "New"},
      "key": {"Char": "n"},
      "modifiers": "CONTROL",
      "context": "Global", 
      "description": "Create new session"
    }
  }
}
```

## Future Enhancements
- Macro recording and playback
- Context-sensitive help for keybindings
- Keybinding analytics (most used shortcuts)
- Gesture support for touchpad users
- Voice command integration
- Keybinding sharing community