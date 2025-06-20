# Task: Add IDE Integration and Editor Extensions

## Priority: Low
## Estimated Effort: 5-7 days
## Dependencies: Plugin system (recommended)

## Overview
Create integrations with popular IDEs and editors to allow seamless interaction with TermAI directly from development environments. This addresses the workflow integration need by bringing AI assistance into the developer's primary workspace.

## Requirements

### Functional Requirements
1. **VS Code Extension**
   - Send selected code to TermAI
   - Insert AI responses at cursor
   - Start TermAI sessions from editor
   - Code review mode
   - Git diff analysis

2. **Neovim Plugin**
   - Vim commands for TermAI interaction
   - Visual mode integration
   - Buffer content sharing
   - Terminal integration

3. **Language Server Protocol (LSP)**
   - Code actions powered by TermAI
   - Hover information with AI insights
   - Diagnostic suggestions
   - Refactoring assistance

4. **Universal Features**
   - Context-aware prompts
   - Project-wide analysis
   - Code documentation generation
   - Error explanation and fixes

### Technical Requirements
1. **Communication Protocol**
   ```rust
   // IPC protocol for editor communication
   #[derive(Serialize, Deserialize)]
   pub enum EditorRequest {
       SendText { content: String, context: EditorContext },
       StartSession { project_path: Option<String> },
       GetResponse { session_id: String, timeout: Duration },
       AnalyzeCode { code: String, language: String, action: CodeAction },
   }
   
   #[derive(Serialize, Deserialize)]
   pub enum EditorResponse {
       SessionCreated { session_id: String },
       TextResponse { content: String, metadata: ResponseMetadata },
       CodeSuggestion { original: String, suggested: String, explanation: String },
       Error { message: String },
   }
   
   #[derive(Serialize, Deserialize)]
   pub struct EditorContext {
       pub file_path: Option<String>,
       pub language: Option<String>,
       pub cursor_position: Option<Position>,
       pub selection: Option<Range>,
       pub project_root: Option<String>,
   }
   ```

2. **TermAI Service Interface**
   ```rust
   pub struct EditorService {
       session_service: Arc<SessionService>,
       llm_service: Arc<LLMService>,
       active_sessions: HashMap<String, String>, // editor_id -> session_id
   }
   
   impl EditorService {
       pub async fn handle_editor_request(&self, request: EditorRequest) -> Result<EditorResponse>;
       pub async fn create_contextual_session(&self, context: EditorContext) -> Result<String>;
       pub async fn analyze_code(&self, code: &str, language: &str, action: CodeAction) -> Result<CodeAnalysis>;
   }
   ```

## Implementation Steps

1. **TermAI Service Extension**
   ```rust
   // Add to main.rs - new service mode
   #[derive(Parser)]
   enum Commands {
       // Existing commands...
       Serve {
           #[arg(long, default_value = "8080")]
           port: u16,
           #[arg(long)]
           socket_path: Option<String>,
       },
   }
   
   // HTTP/WebSocket server for editor communication
   pub struct EditorServer {
       editor_service: Arc<EditorService>,
       port: u16,
   }
   
   impl EditorServer {
       pub async fn start(&self) -> Result<()> {
           let app = Router::new()
               .route("/api/session", post(create_session))
               .route("/api/session/:id/message", post(send_message))
               .route("/api/analyze", post(analyze_code))
               .route("/ws", get(websocket_handler))
               .with_state(self.editor_service.clone());
           
           let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;
           axum::serve(listener, app).await?;
           Ok(())
       }
   }
   ```

2. **VS Code Extension**
   ```typescript
   // vscode-extension/src/extension.ts
   import * as vscode from 'vscode';
   import { TermAIClient } from './termai-client';
   
   export function activate(context: vscode.ExtensionContext) {
       const client = new TermAIClient();
       
       // Send selected text to TermAI
       const sendToTermAI = vscode.commands.registerCommand('termai.sendSelection', async () => {
           const editor = vscode.window.activeTextEditor;
           if (!editor) return;
           
           const selection = editor.selection;
           const text = editor.document.getText(selection);
           
           const context = {
               file_path: editor.document.uri.fsPath,
               language: editor.document.languageId,
               cursor_position: {
                   line: selection.start.line,
                   character: selection.start.character
               },
               selection: {
                   start: selection.start,
                   end: selection.end
               },
               project_root: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
           };
           
           try {
               const response = await client.sendText(text, context);
               await showResponsePanel(response);
           } catch (error) {
               vscode.window.showErrorMessage(`TermAI Error: ${error}`);
           }
       });
       
       // Explain error at cursor
       const explainError = vscode.commands.registerCommand('termai.explainError', async () => {
           const editor = vscode.window.activeTextEditor;
           if (!editor) return;
           
           const diagnostics = vscode.languages.getDiagnostics(editor.document.uri);
           const position = editor.selection.active;
           
           const errorAtCursor = diagnostics.find(d => d.range.contains(position));
           if (!errorAtCursor) {
               vscode.window.showInformationMessage('No error found at cursor position');
               return;
           }
           
           const code = editor.document.getText();
           const errorContext = {
               error_message: errorAtCursor.message,
               error_line: errorAtCursor.range.start.line,
               code_context: code,
               language: editor.document.languageId
           };
           
           const response = await client.analyzeCode(code, editor.document.languageId, {
               type: 'explain_error',
               context: errorContext
           });
           
           await showResponsePanel(response);
       });
       
       context.subscriptions.push(sendToTermAI, explainError);
   }
   
   class TermAIClient {
       private baseUrl = 'http://localhost:8080';
       
       async sendText(text: string, context: any): Promise<any> {
           const response = await fetch(`${this.baseUrl}/api/session`, {
               method: 'POST',
               headers: { 'Content-Type': 'application/json' },
               body: JSON.stringify({
                   type: 'SendText',
                   content: text,
                   context: context
               })
           });
           
           return response.json();
       }
       
       async analyzeCode(code: string, language: string, action: any): Promise<any> {
           const response = await fetch(`${this.baseUrl}/api/analyze`, {
               method: 'POST',
               headers: { 'Content-Type': 'application/json' },
               body: JSON.stringify({
                   code,
                   language,
                   action
               })
           });
           
           return response.json();
       }
   }
   ```

3. **Neovim Plugin**
   ```lua
   -- lua/termai/init.lua
   local M = {}
   local api = vim.api
   local fn = vim.fn
   
   local config = {
       server_url = 'http://localhost:8080',
       auto_start = true,
   }
   
   function M.setup(opts)
       config = vim.tbl_extend('force', config, opts or {})
       
       -- Define commands
       api.nvim_create_user_command('TermAISend', function(args)
           M.send_selection()
       end, { range = true })
       
       api.nvim_create_user_command('TermAIExplain', function(args)
           M.explain_code()
       end, {})
       
       api.nvim_create_user_command('TermAIReview', function(args)
           M.review_code()
       end, {})
       
       -- Set up keymaps
       vim.keymap.set('v', '<leader>ts', M.send_selection, { desc = 'Send selection to TermAI' })
       vim.keymap.set('n', '<leader>te', M.explain_code, { desc = 'Explain code with TermAI' })
       vim.keymap.set('n', '<leader>tr', M.review_code, { desc = 'Review code with TermAI' })
   end
   
   function M.send_selection()
       local start_pos = fn.getpos("'<")
       local end_pos = fn.getpos("'>")
       local lines = api.nvim_buf_get_lines(0, start_pos[2] - 1, end_pos[2], false)
       
       local text = table.concat(lines, '\n')
       local context = {
           file_path = api.nvim_buf_get_name(0),
           language = api.nvim_buf_get_option(0, 'filetype'),
           cursor_position = {
               line = start_pos[2] - 1,
               character = start_pos[3] - 1
           },
           project_root = fn.getcwd()
       }
       
       M.send_request('SendText', {
           content = text,
           context = context
       }, function(response)
           M.show_response(response)
       end)
   end
   
   function M.send_request(request_type, data, callback)
       local curl = require('plenary.curl')
       
       curl.post(config.server_url .. '/api/session', {
           body = vim.json.encode({
               type = request_type,
               data = data
           }),
           headers = {
               ['Content-Type'] = 'application/json'
           },
           callback = function(response)
               if response.status == 200 then
                   local result = vim.json.decode(response.body)
                   callback(result)
               else
                   print('TermAI Error: ' .. response.status)
               end
           end
       })
   end
   
   function M.show_response(response)
       -- Create a floating window with the response
       local buf = api.nvim_create_buf(false, true)
       local lines = vim.split(response.content, '\n')
       api.nvim_buf_set_lines(buf, 0, -1, false, lines)
       
       local width = math.min(vim.o.columns - 4, 80)
       local height = math.min(vim.o.lines - 4, #lines + 2)
       
       local win = api.nvim_open_win(buf, true, {
           relative = 'editor',
           width = width,
           height = height,
           col = (vim.o.columns - width) / 2,
           row = (vim.o.lines - height) / 2,
           style = 'minimal',
           border = 'rounded',
           title = 'TermAI Response'
       })
       
       -- Set buffer options
       api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')
       api.nvim_buf_set_option(buf, 'filetype', 'markdown')
       
       -- Close on escape
       api.nvim_buf_set_keymap(buf, 'n', '<Esc>', '<cmd>close<CR>', { noremap = true })
   end
   
   return M
   ```

4. **Language Server Integration**
   ```rust
   // LSP server implementation
   use tower_lsp::{LspService, Server};
   use tower_lsp::jsonrpc::Result;
   use tower_lsp::lsp_types::*;
   
   struct TermAILanguageServer {
       editor_service: Arc<EditorService>,
   }
   
   #[tower_lsp::async_trait]
   impl LanguageServer for TermAILanguageServer {
       async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
           Ok(InitializeResult {
               capabilities: ServerCapabilities {
                   code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                   hover_provider: Some(HoverProviderCapability::Simple(true)),
                   ..Default::default()
               },
               ..Default::default()
           })
       }
       
       async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
           let actions = vec![
               CodeActionOrCommand::CodeAction(CodeAction {
                   title: "Explain with TermAI".to_string(),
                   kind: Some(CodeActionKind::QUICKFIX),
                   command: Some(Command {
                       title: "Explain".to_string(),
                       command: "termai.explain".to_string(),
                       arguments: Some(vec![
                           serde_json::to_value(&params.text_document.uri).unwrap(),
                           serde_json::to_value(&params.range).unwrap(),
                       ]),
                   }),
                   ..Default::default()
               }),
               CodeActionOrCommand::CodeAction(CodeAction {
                   title: "Suggest improvements".to_string(),
                   kind: Some(CodeActionKind::REFACTOR),
                   command: Some(Command {
                       title: "Improve".to_string(),
                       command: "termai.improve".to_string(),
                       arguments: Some(vec![
                           serde_json::to_value(&params.text_document.uri).unwrap(),
                           serde_json::to_value(&params.range).unwrap(),
                       ]),
                   }),
                   ..Default::default()
               }),
           ];
           
           Ok(Some(actions))
       }
       
       async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
           // Provide AI-powered hover information
           let uri = &params.text_document_position_params.text_document.uri;
           let position = &params.text_document_position_params.position;
           
           // Get context and analyze with AI
           let hover_content = "AI-powered explanation of the symbol at cursor";
           
           Ok(Some(Hover {
               contents: HoverContents::Scalar(MarkedString::String(hover_content.to_string())),
               range: None,
           }))
       }
   }
   ```

## Testing Requirements
- Integration tests with mock editors
- VS Code extension testing
- Neovim plugin testing
- LSP protocol compliance tests
- Performance tests for editor responsiveness

## Acceptance Criteria
- [ ] VS Code extension works correctly
- [ ] Neovim plugin functions properly
- [ ] LSP server provides useful actions
- [ ] Context is properly extracted from editors
- [ ] Responses are formatted appropriately
- [ ] Error handling is robust
- [ ] Performance doesn't impact editor responsiveness

## Distribution
- VS Code: Publish to VS Code Marketplace
- Neovim: Distribute via package managers (lazy.nvim, packer, etc.)
- LSP: Standalone binary distribution
- Documentation: Comprehensive setup guides

## Future Enhancements
- IntelliJ IDEA plugin
- Emacs integration
- Sublime Text plugin
- JetBrains family support
- GitHub Copilot-style inline suggestions
- Real-time collaborative editing with AI
- Custom editor protocol support