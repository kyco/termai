# Task: Add Theme and UI Customization System

## Priority: Low
## Estimated Effort: 3-4 days  
## Dependencies: None

## Overview
Implement a comprehensive theme and UI customization system to allow users to personalize the appearance of TermAI. This includes color schemes, layout options, and visual preferences.

## Requirements

### Functional Requirements
1. **Color Themes**
   - Built-in themes (Dark, Light, High Contrast, Terminal, Solarized)
   - Custom color scheme creation
   - Per-component color customization
   - Syntax highlighting theme selection
   - Import/export theme files

2. **Layout Customization**
   - Adjustable panel sizes
   - Show/hide components (session list, status bar)
   - Border styles (rounded, sharp, double)
   - Spacing and padding options

3. **Typography Options**
   - Font size scaling (for terminals that support it)
   - Text density (compact, normal, spacious)
   - Code block styling options

### Technical Requirements
1. **Theme Configuration Structure**
   ```rust
   #[derive(Serialize, Deserialize, Clone)]
   pub struct Theme {
       pub name: String,
       pub colors: ColorScheme,
       pub layout: LayoutConfig,
       pub typography: TypographyConfig,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub struct ColorScheme {
       // UI Colors
       pub background: Color,
       pub foreground: Color,
       pub border_normal: Color,
       pub border_focused: Color,
       pub selection: Color,
       pub highlight: Color,
       
       // Semantic Colors
       pub success: Color,
       pub warning: Color,
       pub error: Color,
       pub info: Color,
       
       // Role Colors
       pub user_message: Color,
       pub assistant_message: Color,
       pub system_message: Color,
       
       // Syntax Highlighting
       pub code_background: Color,
       pub code_keyword: Color,
       pub code_string: Color,
       pub code_comment: Color,
       pub code_function: Color,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub struct LayoutConfig {
       pub session_list_width: u16,  // percentage of screen
       pub show_status_bar: bool,
       pub border_type: BorderType,
       pub spacing: SpacingConfig,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum BorderType {
       None,
       Plain,
       Rounded,
       Double,
       Thick,
   }
   ```

2. **Theme Management Service**
   ```rust
   pub struct ThemeService {
       current_theme: Arc<RwLock<Theme>>,
       available_themes: HashMap<String, Theme>,
       config_repo: Arc<ConfigRepository>,
   }
   
   impl ThemeService {
       pub fn load_theme(&mut self, name: &str) -> Result<()>;
       pub fn save_theme(&self, theme: Theme) -> Result<()>;
       pub fn get_current_theme(&self) -> Theme;
       pub fn list_themes(&self) -> Vec<String>;
       pub fn import_theme(&mut self, path: &Path) -> Result<()>;
       pub fn export_theme(&self, name: &str, path: &Path) -> Result<()>;
   }
   ```

## Implementation Steps

1. **Built-in Themes**
   ```rust
   // themes/builtin.rs
   impl Theme {
       pub fn dark() -> Self {
           Theme {
               name: "Dark".to_string(),
               colors: ColorScheme {
                   background: Color::Black,
                   foreground: Color::White,
                   border_normal: Color::DarkGray,
                   border_focused: Color::Yellow,
                   selection: Color::Blue,
                   highlight: Color::Yellow,
                   success: Color::Green,
                   warning: Color::Yellow,
                   error: Color::Red,
                   info: Color::Cyan,
                   user_message: Color::Cyan,
                   assistant_message: Color::White,
                   system_message: Color::DarkGray,
                   code_background: Color::DarkGray,
                   code_keyword: Color::Magenta,
                   code_string: Color::Green,
                   code_comment: Color::DarkGray,
                   code_function: Color::Blue,
               },
               layout: LayoutConfig::default(),
               typography: TypographyConfig::default(),
           }
       }
       
       pub fn light() -> Self {
           // Light theme implementation
       }
       
       pub fn high_contrast() -> Self {
           // High contrast theme for accessibility
       }
   }
   ```

2. **Theme Editor UI**
   ```rust
   pub struct ThemeEditor {
       theme: Theme,
       selected_property: usize,
       color_picker: ColorPicker,
       preview_enabled: bool,
   }
   
   impl ThemeEditor {
       pub fn new(theme: Theme) -> Self;
       pub fn select_next_property(&mut self);
       pub fn edit_current_property(&mut self);
       pub fn preview_changes(&self) -> Theme;
       pub fn save_changes(&mut self) -> Result<()>;
   }
   ```

3. **Color Picker Widget**
   ```rust
   pub struct ColorPicker {
       current_color: Color,
       color_mode: ColorMode,
       rgb_values: (u8, u8, u8),
       named_colors: Vec<(String, Color)>,
   }
   
   pub enum ColorMode {
       Named,
       RGB,
       Hex,
   }
   
   impl ColorPicker {
       pub fn new(initial_color: Color) -> Self;
       pub fn set_rgb(&mut self, r: u8, g: u8, b: u8);
       pub fn get_selected_color(&self) -> Color;
   }
   ```

4. **Dynamic Style Application**
   ```rust
   // ui/styles.rs
   pub struct StyleManager {
       theme: Arc<Theme>,
   }
   
   impl StyleManager {
       pub fn get_border_style(&self, focused: bool) -> Style {
           let color = if focused {
               self.theme.colors.border_focused
           } else {
               self.theme.colors.border_normal
           };
           Style::default().fg(color)
       }
       
       pub fn get_message_style(&self, role: MessageRole) -> Style {
           let color = match role {
               MessageRole::User => self.theme.colors.user_message,
               MessageRole::Assistant => self.theme.colors.assistant_message,
               MessageRole::System => self.theme.colors.system_message,
           };
           Style::default().fg(color)
       }
       
       pub fn get_code_block_style(&self) -> Style {
           Style::default()
               .bg(self.theme.colors.code_background)
               .fg(self.theme.colors.foreground)
       }
   }
   ```

5. **Theme Settings Integration**
   ```rust
   // In settings UI
   fn draw_theme_settings(f: &mut Frame, app: &App, area: Rect) {
       let current_theme = app.theme_service.get_current_theme();
       let available_themes = app.theme_service.list_themes();
       
       let theme_list: Vec<ListItem> = available_themes.iter()
           .map(|name| {
               let marker = if name == &current_theme.name { "● " } else { "○ " };
               ListItem::new(format!("{}{}", marker, name))
           })
           .collect();
       
       let themes = List::new(theme_list)
           .block(Block::default().title("Themes").borders(Borders::ALL))
           .highlight_style(Style::default().bg(Color::Blue));
       
       f.render_stateful_widget(themes, area, &mut app.theme_selection_state);
   }
   ```

## Testing Requirements
- Unit tests for theme loading/saving
- UI tests for theme editor
- Performance tests for theme switching
- Color contrast validation tests
- Theme export/import tests

## Acceptance Criteria
- [ ] Multiple built-in themes available
- [ ] Users can switch themes in settings
- [ ] Theme changes apply immediately
- [ ] Custom themes can be created and saved
- [ ] Themes can be exported/imported
- [ ] High contrast theme meets accessibility standards
- [ ] Theme switching is performant (<100ms)

## Built-in Themes

### 1. Dark Theme (Default)
- Background: Black
- Text: White
- Borders: Dark Gray / Yellow (focused)
- Code blocks: Dark Gray background

### 2. Light Theme
- Background: White
- Text: Black
- Borders: Light Gray / Blue (focused)
- Code blocks: Light Gray background

### 3. High Contrast
- Background: Black
- Text: White
- Borders: White / Yellow (focused)
- Strong color contrasts for accessibility

### 4. Solarized Dark
- Based on the popular Solarized color scheme
- Warm, muted colors

### 5. Terminal Classic
- Green text on black background
- Monospace aesthetic
- Minimal colors

## Configuration File Format
```json
{
  "name": "Custom Dark",
  "colors": {
    "background": "#000000",
    "foreground": "#ffffff",
    "border_normal": "#808080",
    "border_focused": "#ffff00",
    "selection": "#0000ff",
    "highlight": "#ffff00",
    "user_message": "#00ffff",
    "assistant_message": "#ffffff",
    "system_message": "#808080"
  },
  "layout": {
    "session_list_width": 25,
    "show_status_bar": true,
    "border_type": "Rounded"
  }
}
```

## Future Enhancements
- Theme marketplace/sharing
- Dynamic themes (time-based)
- Terminal-specific optimizations
- Accessibility theme generator
- Theme inheritance system
- Live theme preview
- CSS-like theme definition language