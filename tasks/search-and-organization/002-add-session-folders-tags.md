# Task: Add Session Folders and Tagging System

## Priority: Medium
## Estimated Effort: 5-7 days
## Dependencies: None

## Overview
Implement a folder and tagging system to help users organize their sessions by project, topic, or any custom categorization. This addresses the pain point of managing many sessions over time.

## Requirements

### Functional Requirements
1. **Folder System**
   - Create, rename, and delete folders
   - Move sessions between folders
   - Nest folders (optional)
   - Default "All Sessions" view
   - Folder icons in session list

2. **Tagging System**
   - Add multiple tags to sessions
   - Create custom tags with colors
   - Filter by single or multiple tags
   - Tag autocomplete
   - Quick tag assignment shortcuts

3. **UI Updates**
   - Folder tree view in left sidebar
   - Tag filter bar
   - Bulk selection for moving/tagging
   - Visual indicators for folders/tags

### Technical Requirements
1. **Database Schema Updates**
   ```sql
   CREATE TABLE folders (
       id TEXT PRIMARY KEY,
       name TEXT NOT NULL,
       parent_id TEXT,
       created_at INTEGER NOT NULL,
       FOREIGN KEY (parent_id) REFERENCES folders(id)
   );
   
   CREATE TABLE tags (
       id TEXT PRIMARY KEY,
       name TEXT NOT NULL UNIQUE,
       color TEXT,
       created_at INTEGER NOT NULL
   );
   
   CREATE TABLE session_tags (
       session_id TEXT NOT NULL,
       tag_id TEXT NOT NULL,
       PRIMARY KEY (session_id, tag_id),
       FOREIGN KEY (session_id) REFERENCES sessions(id),
       FOREIGN KEY (tag_id) REFERENCES tags(id)
   );
   
   -- Add folder_id to sessions table
   ALTER TABLE sessions ADD COLUMN folder_id TEXT REFERENCES folders(id);
   ```

2. **Repository Layer**
   - Create `FolderRepository` and `TagRepository`
   - Update `SessionRepository` with folder/tag methods
   - Implement cascade delete for folders

3. **Service Layer**
   - `OrganizationService` for folder/tag operations
   - Validation logic (unique names, circular references)
   - Bulk operations support

## Implementation Steps

1. **Database Migration**
   - Create migration script for new tables
   - Add indexes for performance
   - Migrate existing sessions to root folder

2. **Backend Services**
   ```rust
   // services/organization_service.rs
   pub struct OrganizationService {
       folder_repo: Arc<FolderRepository>,
       tag_repo: Arc<TagRepository>,
       session_repo: Arc<SessionRepository>,
   }
   ```

3. **UI Components**
   - Folder tree widget
   - Tag selection widget
   - Bulk action toolbar
   - Context menus for folders/tags

4. **Keybindings**
   - `Ctrl+Shift+N`: New folder
   - `Ctrl+T`: Add tag to session
   - `Ctrl+M`: Move session to folder
   - `Ctrl+B`: Bulk select mode

## Testing Requirements
- Unit tests for folder/tag operations
- Integration tests for organization workflows
- UI tests for tree navigation
- Performance tests with deep folder structures

## Acceptance Criteria
- [ ] Users can create and manage folders
- [ ] Sessions can be moved between folders
- [ ] Tags can be created and assigned
- [ ] Filter view by folder and/or tags
- [ ] Bulk operations work correctly
- [ ] UI remains responsive with many folders/tags

## Migration Plan
1. Add new tables without breaking existing functionality
2. Provide migration wizard on first run
3. Option to auto-organize by date
4. Preserve all existing sessions

## Future Enhancements
- Smart folders (saved searches)
- Tag hierarchies
- Folder templates
- Auto-tagging based on content
- Folder/tag sharing between users