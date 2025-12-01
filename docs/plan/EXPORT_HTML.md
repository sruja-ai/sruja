# HTML Export System - Sruja Language

## Overview
This document specifies the HTML export functionality for Sruja language, including templates, CLI commands, and CDN integration for standalone deployment.

## HTML Template System

### Base Template Structure
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}} - Sruja Language Learning</title>
    
    <!-- CSS Dependencies -->
    <link rel="stylesheet" href="{{cdn_url}}/sruja-viewer.css">
    <link rel="stylesheet" href="{{cdn_url}}/themes/{{theme}}.css">
    
    <!-- Custom Styles -->
    <style>
        {{custom_css}}
    </style>
</head>
<body>
    <div id="sruja-container" class="sruja-viewer">
        <header class="sruja-header">
            <h1>{{title}}</h1>
            <p class="sruja-description">{{description}}</p>
        </header>
        
        <main class="sruja-content">
            <div id="sruja-exercises" data-sruja-data='{{json_data}}'></div>
        </main>
        
        <footer class="sruja-footer">
            <div class="sruja-progress">
                <span id="progress-text">Progress: 0%</span>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: 0%"></div>
                </div>
            </div>
        </footer>
    </div>
    
    <!-- JavaScript Dependencies -->
    <script src="{{cdn_url}}/sruja-viewer.min.js"></script>
    <script>
        // Initialize Sruja Viewer
        document.addEventListener('DOMContentLoaded', function() {
            const viewer = new SrujaViewer({
                container: '#sruja-exercises',
                data: {{json_data}},
                theme: '{{theme}}',
                interactive: {{interactive}},
                onProgress: function(progress) {
                    document.getElementById('progress-text').textContent = `Progress: ${progress}%`;
                    document.querySelector('.progress-fill').style.width = `${progress}%`;
                },
                onComplete: function() {
                    console.log('Exercise completed!');
                    // Custom completion handler
                }
            });
            
            viewer.init();
        });
    </script>
</body>
</html>
```

### Template Variables
| Variable | Type | Description | Default |
|----------|------|-------------|---------|
| `{{title}}` | String | Page title | "Sruja Exercise" |
| `{{description}}` | String | Page description | "" |
| `{{json_data}}` | JSON | Sruja AST data | {} |
| `{{theme}}` | String | Theme name | "default" |
| `{{cdn_url}}` | String | CDN base URL | "https://cdn.sruja-lang.org" |
| `{{custom_css}}` | String | Custom CSS | "" |
| `{{interactive}}