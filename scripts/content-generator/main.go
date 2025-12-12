package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"text/template"
	"time"
)

const (
	contentDir   = "apps/website/src/content"
	templatesDir = "scripts/content-generator/templates"
)

type ContentMeta struct {
    Title      string
    Summary    string
    Weight     int
    Type       string
    Date       string
    CourseName string
    ModuleName string
    LessonName string
}

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	command := os.Args[1]

	switch command {
	case "course":
		if len(os.Args) < 3 {
			fmt.Println("Usage: go run scripts/content-generator/main.go course <course-name>")
			os.Exit(1)
		}
		createCourse(os.Args[2])
	case "module":
		if len(os.Args) < 4 {
			fmt.Println("Usage: go run scripts/content-generator/main.go module <course-name> <module-name>")
			os.Exit(1)
		}
		createModule(os.Args[2], os.Args[3])
	case "lesson":
		if len(os.Args) < 5 {
			fmt.Println("Usage: go run scripts/content-generator/main.go lesson <course-name> <module-name> <lesson-name>")
			os.Exit(1)
		}
		createLesson(os.Args[2], os.Args[3], os.Args[4])
	case "tutorial":
		if len(os.Args) < 3 {
			fmt.Println("Usage: go run scripts/content-generator/main.go tutorial <tutorial-name>")
			os.Exit(1)
		}
		createTutorial(os.Args[2])
	case "blog":
		if len(os.Args) < 3 {
			fmt.Println("Usage: go run scripts/content-generator/main.go blog <blog-post-name>")
			os.Exit(1)
		}
		createBlog(os.Args[2])
	case "doc":
		if len(os.Args) < 3 {
			fmt.Println("Usage: go run scripts/content-generator/main.go doc <doc-name>")
			os.Exit(1)
		}
		createDoc(os.Args[2])
	default:
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("Content Generator for Sruja Learn Site")
	fmt.Println()
	fmt.Println("Usage:")
	fmt.Println("  go run scripts/content-generator/main.go <command> [args]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  course <name>              - Create a new course")
	fmt.Println("  module <course> <name>     - Create a new module in a course")
	fmt.Println("  lesson <course> <module> <name> - Create a new lesson")
	fmt.Println("  tutorial <name>            - Create a new tutorial")
	fmt.Println("  blog <name>                - Create a new blog post")
	fmt.Println("  doc <name>                 - Create a new doc page")
	fmt.Println()
	fmt.Println("Examples:")
	fmt.Println("  go run scripts/content-generator/main.go course system-design-301")
	fmt.Println("  go run scripts/content-generator/main.go module system-design-101 advanced-topics")
	fmt.Println("  go run scripts/content-generator/main.go lesson system-design-101 module-1 lesson-6")
	fmt.Println("  go run scripts/content-generator/main.go tutorial advanced-validation")
	fmt.Println("  go run scripts/content-generator/main.go blog introducing-sruja-v2")
}

func createCourse(name string) {
	slug := toSlug(name)
	courseDir := filepath.Join(contentDir, "courses", slug)

	// Check if course already exists
	if _, err := os.Stat(courseDir); err == nil {
		fmt.Printf("❌ Course '%s' already exists at %s\n", name, courseDir)
		os.Exit(1)
	}

	// Create directory
	if err := os.MkdirAll(courseDir, 0o755); err != nil {
		fmt.Printf("❌ Error creating course directory: %v\n", err)
		os.Exit(1)
	}

	// Get title
	title := promptString("Course title", name)

	fmt.Printf("✅ Created course '%s' at %s\n", title, courseDir)
	fmt.Printf("   Next steps:\n")
	fmt.Printf("   1. Create modules: go run scripts/content-generator/main.go module %s <module-name>\n", slug)
	fmt.Printf("   Note: Courses don't need index files in Astro - modules are organized in subdirectories\n")
}

func createModule(course, name string) {
	courseSlug := toSlug(course)
	moduleSlug := toSlug(name)

	courseDir := filepath.Join(contentDir, "courses", courseSlug)
	moduleDir := filepath.Join(courseDir, moduleSlug)

	// Check if course exists
	if _, err := os.Stat(courseDir); os.IsNotExist(err) {
		fmt.Printf("❌ Course '%s' does not exist. Create it first.\n", course)
		os.Exit(1)
	}

	// Check if module already exists
	if _, err := os.Stat(moduleDir); err == nil {
		fmt.Printf("❌ Module '%s' already exists at %s\n", name, moduleDir)
		os.Exit(1)
	}

	// Create directory
	if err := os.MkdirAll(moduleDir, 0o755); err != nil {
		fmt.Printf("❌ Error creating module directory: %v\n", err)
		os.Exit(1)
	}

	// Get metadata
	title := promptString("Module title", name)
	summary := promptString("Module summary", "Learn about "+title+".")
	weight := promptInt("Weight (for ordering)", 1)

	meta := ContentMeta{
		Title:      title,
		Summary:    summary,
		Weight:     weight,
		CourseName: courseSlug,
		ModuleName: moduleSlug,
	}

	// Create module-overview.md
	indexPath := filepath.Join(moduleDir, "module-overview.md")
	createFromTemplate("module_index.md", indexPath, &meta)

	fmt.Printf("✅ Created module '%s' at %s\n", title, moduleDir)
	fmt.Printf("   Next steps:\n")
	fmt.Printf("   1. Edit: %s\n", indexPath)
	fmt.Printf("   2. Create lessons: go run scripts/content-generator/main.go lesson %s %s <lesson-name>\n", courseSlug, moduleSlug)
}

func createLesson(course, module, name string) {
	courseSlug := toSlug(course)
	moduleSlug := toSlug(module)
	lessonSlug := toSlug(name)

	courseDir := filepath.Join(contentDir, "courses", courseSlug)
	moduleDir := filepath.Join(courseDir, moduleSlug)
	lessonFile := filepath.Join(moduleDir, lessonSlug+".md")

	// Check if module exists
	if _, err := os.Stat(moduleDir); os.IsNotExist(err) {
		fmt.Printf("❌ Module '%s/%s' does not exist. Create it first.\n", course, module)
		os.Exit(1)
	}

	// Check if lesson already exists
	if _, err := os.Stat(lessonFile); err == nil {
		fmt.Printf("❌ Lesson '%s' already exists at %s\n", name, lessonFile)
		os.Exit(1)
	}

	// Get metadata
	title := promptString("Lesson title", name)
	summary := promptString("Lesson summary", "Learn about "+title+".")
	weight := promptInt("Weight (for ordering)", 1)

	meta := ContentMeta{
		Title:      title,
		Summary:    summary,
		Weight:     weight,
		CourseName: courseSlug,
		ModuleName: moduleSlug,
		LessonName: lessonSlug,
	}

	// Create lesson.md
	createFromTemplate("lesson.md", lessonFile, &meta)

	fmt.Printf("✅ Created lesson '%s' at %s\n", title, lessonFile)
	fmt.Printf("   Next steps:\n")
	fmt.Printf("   1. Edit: %s\n", lessonFile)
	fmt.Printf("   2. Update module module-overview.md to link to this lesson\n")
}

func createTutorial(name string) {
	slug := toSlug(name)
	tutorialFile := filepath.Join(contentDir, "tutorials", slug+".md")

	// Check if tutorial already exists
	if _, err := os.Stat(tutorialFile); err == nil {
		fmt.Printf("❌ Tutorial '%s' already exists at %s\n", name, tutorialFile)
		os.Exit(1)
	}

	// Get metadata
	title := promptString("Tutorial title", name)
	summary := promptString("Tutorial summary", "Learn how to "+title+".")
	weight := promptInt("Weight (for ordering)", 10)

	meta := ContentMeta{
		Title:   title,
		Summary: summary,
		Weight:  weight,
	}

	// Create tutorial.md
	createFromTemplate("tutorial.md", tutorialFile, &meta)

	fmt.Printf("✅ Created tutorial '%s' at %s\n", title, tutorialFile)
	fmt.Printf("   Next steps: Edit %s\n", tutorialFile)
}

func createBlog(name string) {
    slug := toSlug(name)
    blogFile := filepath.Join(contentDir, "blog", slug+".md")

	// Check if blog already exists
	if _, err := os.Stat(blogFile); err == nil {
		fmt.Printf("❌ Blog post '%s' already exists at %s\n", name, blogFile)
		os.Exit(1)
	}

	// Get metadata
	title := promptString("Blog post title", name)
    summary := promptString("Blog post summary", "")
    date := time.Now().Format("2006-01-02")

    meta := ContentMeta{
        Title:   title,
        Summary: summary,
        Date:    date,
    }

    // Create blog.md
    createFromTemplate("blog.md", blogFile, &meta)

	fmt.Printf("✅ Created blog post '%s' at %s\n", title, blogFile)
	fmt.Printf("   Next steps:\n")
	fmt.Printf("   1. Edit: %s\n", blogFile)
    fmt.Printf("   2. Ensure pubDate and tags are set appropriately\n")
}

func createDoc(name string) {
	slug := toSlug(name)
	docFile := filepath.Join(contentDir, "docs", slug+".md")

	// Check if doc already exists
	if _, err := os.Stat(docFile); err == nil {
		fmt.Printf("❌ Doc '%s' already exists at %s\n", name, docFile)
		os.Exit(1)
	}

	// Get metadata
	title := promptString("Doc title", name)
	summary := promptString("Doc summary", "")
	weight := promptInt("Weight (for ordering)", 10)

	meta := ContentMeta{
		Title:   title,
		Summary: summary,
		Weight:  weight,
	}

	// Create doc.md
	createFromTemplate("doc.md", docFile, &meta)

	fmt.Printf("✅ Created doc '%s' at %s\n", title, docFile)
	fmt.Printf("   Next steps: Edit %s\n", docFile)
}

// Helper functions

func toSlug(s string) string {
	s = strings.ToLower(s)
	s = strings.ReplaceAll(s, " ", "-")
	s = strings.ReplaceAll(s, "_", "-")
	// Remove any non-alphanumeric except hyphens
	var result strings.Builder
	for _, r := range s {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') || r == '-' {
			result.WriteRune(r)
		}
	}
	return strings.Trim(result.String(), "-")
}

func promptString(prompt, defaultValue string) string {
	fmt.Printf("%s [%s]: ", prompt, defaultValue)
	reader := bufio.NewReader(os.Stdin)
	input, _ := reader.ReadString('\n')
	input = strings.TrimSpace(input)
	if input == "" {
		return defaultValue
	}
	return input
}

//nolint:unparam // prompt parameter allows function to be reusable
func promptInt(prompt string, defaultValue int) int {
	fmt.Printf("%s [%d]: ", prompt, defaultValue)
	reader := bufio.NewReader(os.Stdin)
	input, _ := reader.ReadString('\n')
	input = strings.TrimSpace(input)
	if input == "" {
		return defaultValue
	}
	var result int
	if _, err := fmt.Sscanf(input, "%d", &result); err != nil {
		return defaultValue
	}
	return result
}


func createFromTemplate(templateName, outputPath string, meta *ContentMeta) {
	templatePath := filepath.Join(templatesDir, templateName)

	// Read template
	tmplContent, err := os.ReadFile(templatePath)
	if err != nil {
		fmt.Printf("❌ Error reading template %s: %v\n", templatePath, err)
		fmt.Printf("   Creating basic file instead...\n")
		createBasicFile(outputPath, meta)
		return
	}

	// Parse template
	tmpl, err := template.New(templateName).Parse(string(tmplContent))
	if err != nil {
		fmt.Printf("❌ Error parsing template: %v\n", err)
		fmt.Printf("   Creating basic file instead...\n")
		createBasicFile(outputPath, meta)
		return
	}

	// Create output file
	file, err := os.Create(outputPath)
	if err != nil {
		fmt.Printf("❌ Error creating file %s: %v\n", outputPath, err)
		os.Exit(1)
	}
	defer func() {
		if err := file.Close(); err != nil {
			fmt.Printf("⚠️  Warning: error closing file: %v\n", err)
		}
	}()

	// Execute template
	if err := tmpl.Execute(file, meta); err != nil {
		fmt.Printf("❌ Error executing template: %v\n", err)
		//nolint:gocritic // exitAfterDefer: CLI tool needs to exit on error
		os.Exit(1)
	}
}

func createBasicFile(outputPath string, meta *ContentMeta) {
	file, err := os.Create(outputPath)
	if err != nil {
		fmt.Printf("❌ Error creating file: %v\n", err)
		os.Exit(1)
	}
	defer func() {
		if err := file.Close(); err != nil {
			fmt.Printf("⚠️  Warning: error closing file: %v\n", err)
		}
	}()

	if _, err := fmt.Fprintf(file, "---\n"); err != nil {
		fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
	}
	if _, err := fmt.Fprintf(file, "title: %q\n", meta.Title); err != nil {
		fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
	}
	if meta.Summary != "" {
		if _, err := fmt.Fprintf(file, "summary: %q\n", meta.Summary); err != nil {
			fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
		}
	}
	if meta.Weight > 0 {
		if _, err := fmt.Fprintf(file, "weight: %d\n", meta.Weight); err != nil {
			fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
		}
	}
    if meta.Date != "" {
        if _, err := fmt.Fprintf(file, "pubDate: %q\n", meta.Date); err != nil {
            fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
        }
    }
	if _, err := fmt.Fprintf(file, "---\n\n"); err != nil {
		fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
	}
	if _, err := fmt.Fprintf(file, "# %s\n\n", meta.Title); err != nil {
		fmt.Printf("⚠️  Warning: error writing to file: %v\n", err)
	}
}
