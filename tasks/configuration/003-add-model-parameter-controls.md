# Task: Add Model Parameter Controls

## Priority: Medium
## Estimated Effort: 2-3 days
## Dependencies: None

## Overview
Add fine-grained control over AI model parameters (temperature, max tokens, top-p, etc.) at both global and per-session levels. This allows power users to optimize AI responses for different use cases and contexts.

## Requirements

### Functional Requirements
1. **Parameter Controls**
   - Temperature (creativity/randomness)
   - Max tokens (response length)
   - Top-p (nucleus sampling)
   - Frequency penalty (repetition reduction)
   - Presence penalty (topic diversity)
   - System message customization

2. **Configuration Levels**
   - Global defaults
   - Per-session overrides
   - Temporary adjustments for single requests
   - Template-based presets

3. **UI Integration**
   - Parameter sliders in settings
   - Quick adjustment shortcuts
   - Parameter presets (Creative, Balanced, Precise)
   - Real-time parameter validation

### Technical Requirements
1. **Model Parameters Structure**
   ```rust
   #[derive(Serialize, Deserialize, Clone, Debug)]
   pub struct ModelParameters {
       pub temperature: f32,           // 0.0 - 2.0
       pub max_tokens: Option<u32>,    // 1 - model_limit
       pub top_p: f32,                 // 0.0 - 1.0
       pub frequency_penalty: f32,     // -2.0 - 2.0
       pub presence_penalty: f32,      // -2.0 - 2.0
       pub stop_sequences: Vec<String>,
       pub system_message: Option<String>,
   }
   
   impl Default for ModelParameters {
       fn default() -> Self {
           Self {
               temperature: 0.7,
               max_tokens: Some(2048),
               top_p: 1.0,
               frequency_penalty: 0.0,
               presence_penalty: 0.0,
               stop_sequences: vec![],
               system_message: None,
           }
       }
   }
   
   impl ModelParameters {
       pub fn creative() -> Self {
           Self {
               temperature: 1.2,
               top_p: 0.9,
               ..Default::default()
           }
       }
       
       pub fn balanced() -> Self {
           Default::default()
       }
       
       pub fn precise() -> Self {
           Self {
               temperature: 0.2,
               top_p: 0.8,
               frequency_penalty: 0.1,
               ..Default::default()
           }
       }
   }
   ```

2. **Parameter Management Service**
   ```rust
   pub struct ParameterService {
       global_params: ModelParameters,
       session_params: HashMap<String, ModelParameters>,
       presets: HashMap<String, ModelParameters>,
       config_repo: Arc<ConfigRepository>,
   }
   
   impl ParameterService {
       pub fn get_params_for_session(&self, session_id: &str) -> ModelParameters;
       pub fn set_session_params(&mut self, session_id: &str, params: ModelParameters);
       pub fn get_global_params(&self) -> &ModelParameters;
       pub fn set_global_params(&mut self, params: ModelParameters);
       pub fn load_preset(&self, name: &str) -> Option<ModelParameters>;
       pub fn save_preset(&mut self, name: String, params: ModelParameters);
       pub fn validate_params(&self, params: &ModelParameters) -> Result<()>;
   }
   ```

3. **Database Schema Updates**
   ```sql
   -- Add parameters column to sessions table
   ALTER TABLE sessions ADD COLUMN parameters TEXT; -- JSON serialized ModelParameters
   
   -- Add global parameters to config
   INSERT OR REPLACE INTO config (key, value) VALUES ('global_model_params', ?);
   
   -- Parameter presets table
   CREATE TABLE parameter_presets (
       name TEXT PRIMARY KEY,
       parameters TEXT NOT NULL, -- JSON serialized ModelParameters
       description TEXT,
       created_at INTEGER NOT NULL
   );
   ```

## Implementation Steps

1. **Extend LLM Adapters**
   ```rust
   // Update OpenAI adapter to use parameters
   impl OpenAIAdapter {
       async fn complete_with_params(
           &self,
           prompt: &str,
           params: &ModelParameters,
       ) -> Result<CompletionResponse> {
           let request = ChatCompletionRequest {
               model: self.config.model.clone(),
               messages: vec![ChatMessage {
                   role: "user".to_string(),
                   content: prompt.to_string(),
               }],
               temperature: Some(params.temperature),
               max_tokens: params.max_tokens,
               top_p: Some(params.top_p),
               frequency_penalty: Some(params.frequency_penalty),
               presence_penalty: Some(params.presence_penalty),
               stop: if params.stop_sequences.is_empty() { 
                   None 
               } else { 
                   Some(params.stop_sequences.clone()) 
               },
               ..Default::default()
           };
           
           // Add system message if present
           if let Some(ref system_msg) = params.system_message {
               request.messages.insert(0, ChatMessage {
                   role: "system".to_string(),
                   content: system_msg.clone(),
               });
           }
           
           self.send_request(request).await
       }
   }
   ```

2. **Parameter Controls UI**
   ```rust
   pub struct ParameterControls {
       params: ModelParameters,
       selected_param: usize,
       preset_selection: Option<String>,
       editing: bool,
   }
   
   impl ParameterControls {
       pub fn new(params: ModelParameters) -> Self;
       pub fn next_parameter(&mut self);
       pub fn adjust_current_parameter(&mut self, delta: f32);
       pub fn load_preset(&mut self, name: &str, presets: &HashMap<String, ModelParameters>);
       pub fn get_parameters(&self) -> ModelParameters;
   }
   ```

3. **Parameter Editor Widget**
   ```rust
   fn draw_parameter_controls(f: &mut Frame, controls: &mut ParameterControls, area: Rect) {
       let chunks = Layout::default()
           .direction(Direction::Vertical)
           .constraints([
               Constraint::Length(3), // Preset selection
               Constraint::Min(0),    // Parameter sliders
           ])
           .split(area);
       
       // Preset selection
       let presets = vec!["Creative", "Balanced", "Precise", "Custom"];
       let preset_tabs = Tabs::new(presets)
           .style(Style::default().fg(Color::White))
           .highlight_style(Style::default().fg(Color::Yellow))
           .select(controls.get_selected_preset_index());
       
       f.render_widget(preset_tabs, chunks[0]);
       
       // Parameter sliders
       let param_area = Layout::default()
           .direction(Direction::Vertical)
           .constraints([Constraint::Length(3); 6]) // 6 parameters
           .split(chunks[1]);
       
       draw_parameter_slider(f, "Temperature", controls.params.temperature, 0.0, 2.0, param_area[0]);
       draw_parameter_slider(f, "Max Tokens", controls.params.max_tokens.unwrap_or(2048) as f32, 1.0, 4096.0, param_area[1]);
       draw_parameter_slider(f, "Top-p", controls.params.top_p, 0.0, 1.0, param_area[2]);
       draw_parameter_slider(f, "Frequency Penalty", controls.params.frequency_penalty, -2.0, 2.0, param_area[3]);
       draw_parameter_slider(f, "Presence Penalty", controls.params.presence_penalty, -2.0, 2.0, param_area[4]);
   }
   
   fn draw_parameter_slider(f: &mut Frame, name: &str, value: f32, min: f32, max: f32, area: Rect) {
       let percentage = ((value - min) / (max - min) * 100.0) as u16;
       let gauge = Gauge::default()
           .block(Block::default().title(format!("{}: {:.2}", name, value)).borders(Borders::ALL))
           .gauge_style(Style::default().fg(Color::Yellow))
           .percent(percentage);
       
       f.render_widget(gauge, area);
   }
   ```

4. **Session Parameter Integration**
   ```rust
   // In session management
   impl SessionService {
       pub async fn send_message_with_params(
           &self,
           session_id: &str,
           message: &str,
           params: Option<ModelParameters>,
       ) -> Result<String> {
           let effective_params = params.unwrap_or_else(|| {
               self.parameter_service.get_params_for_session(session_id)
           });
           
           // Use parameters in LLM call
           let response = self.llm_adapter.complete_with_params(message, &effective_params).await?;
           
           // Save parameters with session if they're custom
           if params.is_some() {
               self.parameter_service.set_session_params(session_id, effective_params);
           }
           
           Ok(response.content)
       }
   }
   ```

5. **Quick Parameter Adjustment**
   ```rust
   // In event handling
   impl EventHandler {
       fn handle_parameter_shortcuts(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
           if key.modifiers.contains(KeyModifiers::ALT) {
               match key.code {
                   KeyCode::Char('1') => {
                       // Load Creative preset
                       app.load_parameter_preset("Creative");
                   }
                   KeyCode::Char('2') => {
                       // Load Balanced preset
                       app.load_parameter_preset("Balanced");
                   }
                   KeyCode::Char('3') => {
                       // Load Precise preset
                       app.load_parameter_preset("Precise");
                   }
                   KeyCode::Char('+') => {
                       // Increase temperature
                       app.adjust_temperature(0.1);
                   }
                   KeyCode::Char('-') => {
                       // Decrease temperature
                       app.adjust_temperature(-0.1);
                   }
                   _ => {}
               }
           }
           Ok(())
       }
   }
   ```

## Testing Requirements
- Unit tests for parameter validation
- Integration tests for LLM parameter passing
- UI tests for parameter controls
- Preset loading/saving tests
- Parameter persistence tests

## Acceptance Criteria
- [ ] Parameters can be adjusted globally and per-session
- [ ] Built-in presets work correctly
- [ ] Custom presets can be saved and loaded
- [ ] Parameter validation prevents invalid values
- [ ] Parameters persist across app restarts
- [ ] UI controls are intuitive and responsive
- [ ] Quick shortcuts work for common adjustments

## Parameter Descriptions
- **Temperature**: Controls randomness (0.0 = deterministic, 2.0 = very creative)
- **Max Tokens**: Maximum response length (varies by model)
- **Top-p**: Nucleus sampling threshold (0.9 = focused, 1.0 = full vocabulary)
- **Frequency Penalty**: Reduces repetition of tokens (-2.0 to 2.0)
- **Presence Penalty**: Encourages new topics (-2.0 to 2.0)
- **Stop Sequences**: Tokens that stop generation early

## Built-in Presets
1. **Creative**: High temperature, lower top-p for imaginative responses
2. **Balanced**: Default parameters for general use
3. **Precise**: Low temperature, high top-p for factual responses
4. **Code**: Optimized for code generation and technical content
5. **Writing**: Optimized for creative writing and storytelling

## Future Enhancements
- A/B testing of parameter sets
- Parameter recommendation based on conversation type
- Automatic parameter tuning based on user feedback
- Parameter analytics and optimization
- Community-shared presets
- Context-aware parameter suggestions