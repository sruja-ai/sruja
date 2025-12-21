# JSON Schema - Sruja Language AST

## Overview
This document defines the exact field mappings from the Abstract Syntax Tree (AST) for the Sruja language, including examples and validation rules.

## Core AST Node Types

### 1. Program Node
```json
{
  "type": "Program",
  "body": [Statement],
  "sourceType": "module",
  "interpreter": null
}
```

**Fields:**
- `type`: String - Always "Program"
- `body`: Array of Statement nodes
- `sourceType`: String - "module" or "script"
- `interpreter`: String or null - Interpreter directive

### 2. Statement Nodes

#### ExpressionStatement
```json
{
  "type": "ExpressionStatement",
  "expression": Expression,
  "directive": String or null
}
```

#### VariableDeclaration
```json
{
  "type": "VariableDeclaration",
  "declarations": [VariableDeclarator],
  "kind": "var" | "let" | "const"
}
```

#### FunctionDeclaration
```json
{
  "type": "FunctionDeclaration",
  "id": Identifier or null,
  "params": [Pattern],
  "body": BlockStatement,
  "generator": Boolean,
  "async": Boolean
}
```

### 3. Expression Nodes

#### Identifier
```json
{
  "type": "Identifier",
  "name": String,
  "decorators": [Decorator] or null
}
```

#### Literal
```json
{
  "type": "Literal",
  "value": String | Number | Boolean | null,
  "raw": String,
  "regex": {
    "pattern": String,
    "flags": String
  } or null
}
```

#### CallExpression
```json
{
  "type": "CallExpression",
  "callee": Expression,
  "arguments": [Expression | SpreadElement],
  "optional": Boolean
}
```

### 4. Sruja-Specific Nodes

#### LanguageBlock
```json
{
  "type": "LanguageBlock",
  "language": String,
  "content": String,
  "interactive": Boolean,
  "hints": [String],
  "validation": ValidationRule
}
```

#### ExerciseNode
```json
{
  "type": "ExerciseNode",
  "title": String,
  "description": String,
  "difficulty": "beginner" | "intermediate" | "advanced",
  "tasks": [Task],
  "feedback": FeedbackConfig
}
```

#### InteractiveElement
```json
{
  "type": "InteractiveElement",
  "elementType": "input" | "button" | "dragdrop" | "multiple-choice",
  "properties": Object,
  "validation": ValidationRule,
  "feedback": FeedbackConfig
}
```

## Validation Rules

### ValidationRule Schema
```json
{
  "type": "ValidationRule",
  "ruleType": "exact" | "regex" | "range" | "custom",
  "expectedValue": any,
  "errorMessage": String,
  "caseSensitive": Boolean
}
```

### FeedbackConfig Schema
```json
{
  "type": "FeedbackConfig",
  "correct": String,
  "incorrect": String,
  "hint": String,
  "retry": Boolean
}
```

## Examples

### Complete Language Exercise
```json
{
  "type": "Program",
  "body": [
    {
      "type": "LanguageBlock",
      "language": "spanish",
      "content": "Translate: Hello, how are you?",
      "interactive": true,
      "hints": ["Use formal greeting", "Question form needed"],
      "validation": {
        "ruleType": "exact",
        "expectedValue": "Hola, ¿cómo está usted?",
        "errorMessage": "Check your grammar and accents",
        "caseSensitive": false
      }
    },
    {
      "type": "ExerciseNode",
      "title": "Spanish Greetings",
      "description": "Practice formal greetings in Spanish",
      "difficulty": "beginner",
      "tasks": [
        {
          "type": "InteractiveElement",
          "elementType": "input",
          "properties": {
            "placeholder": "Type your answer here",
            "maxLength": 100
          },
          "validation": {
            "ruleType": "regex",
            "expectedValue": "^[Hh]ola.*",
            "errorMessage": "Answer must start with 'Hola'",
            "caseSensitive": false
          },
          "feedback": {
            "correct": "¡Excelente! You used the correct greeting.",
            "incorrect": "Remember to start with 'Hola'",
            "hint": "Hola means Hello in Spanish",
            "retry": true
          }
        }
      ],
      "feedback": {
        "correct": "Great job! You completed the exercise.",
        "incorrect": "Keep practicing your Spanish greetings.",
        "hint": "Review the lesson on formal greetings",
        "retry": true
      }
    }
  ],
  "sourceType": "module",
  "interpreter": null
}
```

## Field Mapping Reference

| AST Node Type | JSON Path | Description | Required |
|--------------|-----------|-------------|----------|
| Program | `$.type` | Node type identifier | Yes |
| Program | `$.body` | Array of statements | Yes |
| LanguageBlock | `$.language` | Target language code | Yes |
| LanguageBlock | `$.content` | Display content | Yes |
| LanguageBlock | `$.interactive` | Interactive flag | No |
| ExerciseNode | `$.title` | Exercise title | Yes |
| ExerciseNode | `$.difficulty` | Difficulty level | Yes |
| InteractiveElement | `$.elementType` | UI element type | Yes |
| ValidationRule | `$.ruleType` | Validation method | Yes |
| FeedbackConfig | `$.correct` | Success message | Yes |

## Validation Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "type": {"enum": ["Program"]},
    "body": {
      "type": "array",
      "items": {"$ref": "#/definitions/Statement"}
    },
    "sourceType": {"enum": ["module", "script"]},
    "interpreter": {"type": ["string", "null"]}
  },
  "required": ["type", "body", "sourceType"],
  "definitions": {
    "Statement": {
      "oneOf": [
        {"$ref": "#/definitions/LanguageBlock"},
        {"$ref": "#/definitions/ExerciseNode"},
        {"$ref": "#/definitions/ExpressionStatement"}
      ]
    },
    "LanguageBlock": {
      "type": "object",
      "properties": {
        "type": {"enum": ["LanguageBlock"]},
        "language": {"type": "string"},
        "content": {"type": "string"},
        "interactive": {"type": "boolean"},
        "hints": {"type": "array", "items": {"type": "string"}},
        "validation": {"$ref": "#/definitions/ValidationRule"}
      },
      "required": ["type", "language", "content"]
    }
  }
}
```