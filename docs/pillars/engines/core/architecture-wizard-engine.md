# Architecture Wizard Engine

**Status**: Core Engine  
**Pillars**: Core (Guided Creation)

[← Back to Engines](../README.md)

## Overview

The Architecture Wizard Engine provides guided creation flows for building architecture models, helping users create architectures through step-by-step wizards and templates.

**This engine makes architecture creation accessible to all skill levels.**

## Purpose

The Architecture Wizard Engine:

- ✅ Guides users through architecture creation
- ✅ Provides template-based starting points
- ✅ Offers step-by-step wizards
- ✅ Validates inputs during creation
- ✅ Suggests best practices
- ✅ Integrates with Template Engine
- ✅ Supports multiple architecture patterns

## Wizard Types

### 1. Template-Based Wizard
- Browse templates
- Search and filter
- Select template
- Customize
- Instantiate

### 2. Guided Creation Wizard
- Step-by-step questions
- Pattern selection
- Component addition
- Relationship definition
- Validation and review

### 3. AI-Assisted Wizard
- Natural language input
- AI-generated architecture
- Review and refine
- Customize as needed

## Wizard Flow

### Template Selection
1. Browse template categories
2. Search templates
3. Filter by complexity/layout
4. Preview template
5. Select template

### Customization
1. Review template structure
2. Modify components
3. Adjust relationships
4. Add/remove elements
5. Configure properties

### Validation
1. Run validation rules
2. Check for errors
3. Suggest improvements
4. Review warnings

### Completion
1. Review final architecture
2. Save to project
3. Open in editor
4. Continue editing

## Integration Points

### Template Engine
- Template search
- Template instantiation
- Template metadata

### Auto-Layout Engine
- Automatic positioning
- Layout mode selection
- Visual arrangement

### Validation Engine
- Input validation
- Best practice checks
- Error detection

### AI Engines
- Natural language processing
- Architecture generation
- Recommendations

## UI Components

### Template Browser
- Grid view of templates
- Search bar
- Filter sidebar
- Preview panel

### Wizard Steps
- Multi-step forms
- Progress indicator
- Navigation controls
- Help text

### Validation Panel
- Error list
- Warning messages
- Suggestions
- Fix actions

## MCP API

```
wizard.start(type)
wizard.next(step)
wizard.previous(step)
wizard.validate(step)
wizard.complete()
wizard.templates()
wizard.guide(type)
```

## Strategic Value

The Architecture Wizard Engine provides:

- ✅ Lower barrier to entry
- ✅ Guided best practices
- ✅ Faster architecture creation
- ✅ Consistent patterns
- ✅ Learning tool

**This is critical for onboarding and adoption.**

## Implementation Status

✅ Architecture designed  
✅ Wizard flows specified  
✅ Integration points defined  
📋 Implementation in progress

---

*The Architecture Wizard Engine provides guided creation flows for architecture models.*

