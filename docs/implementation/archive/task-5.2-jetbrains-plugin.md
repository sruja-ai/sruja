# Task 5.2: JetBrains Plugin

**Priority**: 🟢 Medium (Enhances developer experience)
**Technology**: Kotlin/Java
**Estimated Time**: 5-7 days
**Dependencies**: Task 1.7 (LSP), Task 4.1 (Studio Core)

## Overview

Create JetBrains plugin (IntelliJ IDEA, WebStorm, etc.) for Sruja that provides:
- LSP integration (syntax highlighting, errors, completion, hover)
- Studio webview (visual editor)
- File operations (read/write `.sruja` files)
- Git integration (VCS status, PR workflows)

## Features

### 1. LSP Integration

**Language Support**:
- Syntax highlighting (`.sruja` files)
- Error diagnostics (red squiggles)
- Code completion
- Hover information
- Go to definition
- Find references
- Quick fixes (intentions)
- Formatting (format on save)

**Implementation**:
- Plugin launches LSP server (`sruja lsp`)
- Communicates via stdio
- Displays diagnostics in Problems tool window

### 2. Studio Webview

**Visual Editor**:
- Embed Local Studio in tool window
- Same UI as CLI Studio
- File operations via IntelliJ APIs
- Real-time sync with editor

**Implementation**:
- Reuse `apps/studio/` React app (from monorepo)
- Adapt for IntelliJ tool window
- Use IntelliJ file system APIs
- Import `@sruja/viewer` and `@sruja/shared` packages

### 3. File Operations

**IntelliJ APIs**:
- Read/write `.sruja` files
- Watch for file changes
- Handle file saves
- Workspace file management

### 4. Git Integration

**VCS Features**:
- Show `.sruja` files in Version Control
- Diff view for changes
- Commit `.sruja` files
- Create PRs (via GitHub integration)

## Plugin Structure

```
jetbrains-plugin/
  ├── src/
  │   ├── main/
  │   │   ├── kotlin/
  │   │   │   ├── SrujaPlugin.kt        # Main plugin class
  │   │   │   ├── lsp/
  │   │   │   │   ├── LspClient.kt     # LSP client
  │   │   │   │   └── LspServer.kt     # LSP server launcher
  │   │   │   ├── studio/
  │   │   │   │   ├── StudioToolWindow.kt  # Studio tool window
  │   │   │   │   └── StudioPanel.kt       # Studio panel
  │   │   │   ├── lang/
  │   │   │   │   ├── SrujaLanguage.kt     # Language definition
  │   │   │   │   ├── SrujaParser.kt       # Parser (optional, or use LSP)
  │   │   │   │   └── SrujaSyntaxHighlighter.kt  # Syntax highlighting
  │   │   │   └── actions/
  │   │   │       ├── OpenStudioAction.kt  # Open Studio action
  │   │   │       ├── FormatAction.kt      # Format action
  │   │   │       └── ValidateAction.kt    # Validate action
  │   │   └── resources/
  │   │       ├── META-INF/
  │   │       │   └── plugin.xml           # Plugin manifest
  │   │       └── studio/
  │   │           └── index.html          # Studio HTML
  │   └── test/
  │       └── kotlin/
  │           └── SrujaPluginTest.kt
  ├── build.gradle.kts
  └── settings.gradle.kts
```

## Implementation

### Plugin Entry Point

```kotlin
// src/main/kotlin/SrujaPlugin.kt
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity

class SrujaPlugin : StartupActivity {
    override fun runActivity(project: Project) {
        // Initialize LSP client
        val lspClient = SrujaLspClient(project)
        lspClient.start()
        
        // Register Studio tool window
        val studioToolWindow = StudioToolWindow(project)
        studioToolWindow.init()
    }
}
```

### LSP Client

```kotlin
// src/main/kotlin/lsp/LspClient.kt
import com.intellij.openapi.project.Project
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.services.*
import org.eclipse.lsp4j.jsonrpc.Launcher

class SrujaLspClient(private val project: Project) {
    private var languageClient: LanguageClient? = null
    
    fun start() {
        val serverProcess = ProcessBuilder("sruja", "lsp").start()
        
        val launcher = Launcher.createLauncher(
            SrujaLanguageClient::class.java,
            LanguageServer::class.java,
            serverProcess.inputStream,
            serverProcess.outputStream
        )
        
        languageClient = launcher.remoteProxy
        launcher.startListening()
    }
    
    fun stop() {
        languageClient?.shutdown()
    }
}
```

### Studio Tool Window

```kotlin
// src/main/kotlin/studio/StudioToolWindow.kt
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import javax.swing.JPanel
import java.awt.BorderLayout

class StudioToolWindow : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = StudioPanel(project)
        val content = ContentFactory.SERVICE.getInstance()
            .createContent(panel, "", false)
        toolWindow.contentManager.addContent(content)
    }
}

class StudioPanel(private val project: Project) : JPanel(BorderLayout()) {
    private val browser: JCEFHtmlPanel
    
    init {
        browser = JCEFHtmlPanel()
        browser.loadHTML(loadStudioHTML())
        
        add(browser, BorderLayout.CENTER)
        
        // Handle messages from Studio
        browser.addMessageHandler { message ->
            when (message.command) {
                "loadFile" -> {
                    val file = VirtualFileManager.getInstance()
                        .findFileByUrl("file://${message.path}")
                    val content = file?.contentsToByteArray()?.toString(Charsets.UTF_8)
                    browser.postMessage("fileLoaded", content)
                }
                "saveFile" -> {
                    val file = VirtualFileManager.getInstance()
                        .findFileByUrl("file://${message.path}")
                    file?.setBinaryContent(message.content.toByteArray(Charsets.UTF_8))
                }
            }
        }
    }
    
    private fun loadStudioHTML(): String {
        // Load Studio React app HTML
        val html = javaClass.getResource("/studio/index.html")?.readText()
        return html ?: "<html><body>Studio not found</body></html>"
    }
}
```

### Plugin Manifest

```xml
<!-- src/main/resources/META-INF/plugin.xml -->
<idea-plugin>
    <id>com.sruja.sruja</id>
    <name>Sruja</name>
    <version>0.1.0</version>
    <vendor>Sruja AI</vendor>
    
    <description>
        Sruja Architecture as Code language support for IntelliJ IDEA
    </description>
    
    <depends>com.intellij.modules.platform</depends>
    <depends>com.intellij.modules.lang</depends>
    
    <extensions defaultExtensionNs="com.intellij">
        <!-- Language -->
        <fileType name="Sruja" implementationClass="com.sruja.lang.SrujaFileType" />
        <lang.parserDefinition language="Sruja" implementationClass="com.sruja.lang.SrujaParserDefinition" />
        <lang.syntaxHighlighterFactory language="Sruja" implementationClass="com.sruja.lang.SrujaSyntaxHighlighterFactory" />
        
        <!-- Tool Window -->
        <toolWindow id="Sruja Studio" anchor="right" factoryClass="com.sruja.studio.StudioToolWindow" />
        
        <!-- Actions -->
        <action id="Sruja.OpenStudio" class="com.sruja.actions.OpenStudioAction" text="Open Studio" />
        <action id="Sruja.Format" class="com.sruja.actions.FormatAction" text="Format Document" />
    </extensions>
    
    <applicationListeners>
        <listener class="com.sruja.SrujaPlugin" topic="com.intellij.openapi.project.ProjectManagerListener" />
    </applicationListeners>
</idea-plugin>
```

## Build Configuration

```kotlin
// build.gradle.kts
plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "1.9.0"
    id("org.jetbrains.intellij") version "1.15.0"
}

intellij {
    version.set("2023.2")
    type.set("IC") // IntelliJ IDEA Community
    plugins.set(listOf("com.intellij.java"))
}

dependencies {
    implementation("org.eclipse.lsp4j:org.eclipse.lsp4j:0.21.0")
    implementation("org.eclipse.lsp4j:org.eclipse.lsp4j.jsonrpc:0.21.0")
}
```

## Acceptance Criteria

- [ ] Plugin installs and activates
- [ ] Syntax highlighting works for `.sruja` files
- [ ] LSP integration works (errors, completion, hover)
- [ ] Studio tool window opens and works
- [ ] File operations work (load/save via IntelliJ APIs)
- [ ] Format document action works
- [ ] Format on save works
- [ ] Quick fixes work (intentions)
- [ ] Go to definition works
- [ ] Find references works
- [ ] Problems tool window shows errors
- [ ] Works with Git (VCS integration)

## Dependencies

- Task 1.7 (LSP) - Language server protocol
- Task 4.1 (Studio Core) - Studio React app
- `org.eclipse.lsp4j` - LSP client library
- IntelliJ Platform SDK

## Notes

- **Reuse Studio**: Embed `local-studio/` React app in tool window
- **LSP Server**: Plugin launches `sruja lsp` command
- **File Operations**: Use IntelliJ VirtualFile APIs
- **Performance**: Lazy load Studio tool window (only when opened)
- **Platform**: Works with IntelliJ IDEA, WebStorm, PyCharm, etc.

