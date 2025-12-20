package main

import (
	"bufio"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCommandCourse(t *testing.T) {
	tmpDir := t.TempDir()
	oldContentDir := contentDir
	contentDir = tmpDir
	defer func() { contentDir = oldContentDir }()

	// Mock templatesDir
	oldTemplatesDir := templatesDir
	templatesDir = filepath.Join(tmpDir, "templates")
	_ = os.MkdirAll(templatesDir, 0755)
	defer func() { templatesDir = oldTemplatesDir }()

	// Input for course title
	defaultReader = bufio.NewReader(strings.NewReader("My Course\n"))

	createCourse("my-course")

	coursePath := filepath.Join(tmpDir, "courses", "my-course")
	if _, err := os.Stat(coursePath); os.IsNotExist(err) {
		t.Errorf("course directory not created at %s", coursePath)
	}
}

func TestCommandModule(t *testing.T) {
	tmpDir := t.TempDir()
	oldContentDir := contentDir
	contentDir = tmpDir
	defer func() { contentDir = oldContentDir }()

	oldTemplatesDir := templatesDir
	templatesDir = filepath.Join(tmpDir, "templates")
	_ = os.MkdirAll(templatesDir, 0755)
	defer func() { templatesDir = oldTemplatesDir }()

	// Prepare course dir
	coursePath := filepath.Join(tmpDir, "courses", "my-course")
	_ = os.MkdirAll(coursePath, 0755)

	// Inputs: title, summary, weight
	defaultReader = bufio.NewReader(strings.NewReader("My Module\nThis is a module\n2\n"))

	createModule("my-course", "my-module")

	modulePath := filepath.Join(coursePath, "my-module")
	if _, err := os.Stat(modulePath); os.IsNotExist(err) {
		t.Errorf("module directory not created at %s", modulePath)
	}
}

func TestCommandTutorial(t *testing.T) {
	tmpDir := t.TempDir()
	oldContentDir := contentDir
	contentDir = tmpDir
	defer func() { contentDir = oldContentDir }()

	oldTemplatesDir := templatesDir
	templatesDir = filepath.Join(tmpDir, "templates")
	_ = os.MkdirAll(templatesDir, 0755)
	defer func() { templatesDir = oldTemplatesDir }()

	_ = os.MkdirAll(filepath.Join(tmpDir, "tutorials"), 0755)

	// Inputs: title, summary, weight
	defaultReader = bufio.NewReader(strings.NewReader("My Tutorial\nHow to use sruja\n1\n"))

	createTutorial("my-tutorial")

	tutorialPath := filepath.Join(tmpDir, "tutorials", "my-tutorial.md")
	if _, err := os.Stat(tutorialPath); os.IsNotExist(err) {
		t.Errorf("tutorial file not created at %s", tutorialPath)
	}
}

func TestPrompts(t *testing.T) {
	// Test promptString with input
	input := "My Custom Title\n"
	defaultReader = bufio.NewReader(strings.NewReader(input))
	got := promptString("Prompt", "Default")
	if got != "My Custom Title" {
		t.Errorf("promptString = %q, want %q", got, "My Custom Title")
	}

	// Test promptString with empty input (default)
	defaultReader = bufio.NewReader(strings.NewReader("\n"))
	got = promptString("Prompt", "Default")
	if got != "Default" {
		t.Errorf("promptString with empty = %q, want %q", got, "Default")
	}

	// Test promptInt with input
	defaultReader = bufio.NewReader(strings.NewReader("42\n"))
	gotInt := promptInt("Prompt", 10)
	if gotInt != 42 {
		t.Errorf("promptInt = %d, want %d", gotInt, 42)
	}

	// Test promptInt with invalid input (default)
	defaultReader = bufio.NewReader(strings.NewReader("abc\n"))
	gotInt = promptInt("Prompt", 10)
	if gotInt != 10 {
		t.Errorf("promptInt with invalid = %d, want %d", gotInt, 10)
	}
}

func TestToSlug(t *testing.T) {
	tests := []struct {
		in  string
		out string
	}{
		{"Hello World", "hello-world"},
		{"System_Design 101", "system-design-101"},
		{"  Advanced__Topics  ", "advanced--topics"},
		{"Intro!", "intro"},
		{"UPPER lower 123", "upper-lower-123"},
	}
	for _, tt := range tests {
		got := toSlug(tt.in)
		if got != tt.out {
			t.Errorf("toSlug(%q) = %q; want %q", tt.in, got, tt.out)
		}
	}
}

func TestCreateBasicFile(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "lesson.md")
	meta := &ContentMeta{Title: "My Title", Summary: "Summary", Weight: 3, Date: "2025-01-02"}
	createBasicFile(out, meta)
	b, err := os.ReadFile(out)
	if err != nil {
		t.Fatalf("read file: %v", err)
	}
	s := string(b)
	if !strings.Contains(s, `title: "My Title"`) {
		t.Errorf("missing title")
	}
	if !strings.Contains(s, `summary: "Summary"`) {
		t.Errorf("missing summary")
	}
	if !strings.Contains(s, "weight: 3") {
		t.Errorf("missing weight")
	}
	if !strings.Contains(s, `pubDate: "2025-01-02"`) {
		t.Errorf("missing pubDate")
	}
	if !strings.Contains(s, "# My Title") {
		t.Errorf("missing header")
	}
}

func TestCreateFromTemplateFallback(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "doc.md")
	meta := &ContentMeta{Title: "Doc Title"}
	createFromTemplate("missing-template.md", out, meta)
	b, err := os.ReadFile(out)
	if err != nil {
		t.Fatalf("read file: %v", err)
	}
	if !strings.Contains(string(b), `title: "Doc Title"`) {
		t.Errorf("basic file not created")
	}
}
func TestPrintUsage(t *testing.T) {
	printUsage()
}
func TestCommandLesson(t *testing.T) {
	tmpDir := t.TempDir()
	oldContentDir := contentDir
	contentDir = tmpDir
	defer func() { contentDir = oldContentDir }()

	oldTemplatesDir := templatesDir
	templatesDir = filepath.Join(tmpDir, "templates")
	_ = os.MkdirAll(templatesDir, 0755)
	defer func() { templatesDir = oldTemplatesDir }()

	// Prepare course/module dir
	modulePath := filepath.Join(tmpDir, "courses", "my-course", "my-module")
	_ = os.MkdirAll(modulePath, 0755)

	// Inputs: title, summary, weight
	defaultReader = bufio.NewReader(strings.NewReader("My Lesson\nThis is a lesson\n5\n"))

	createLesson("my-course", "my-module", "my-lesson")

	lessonPath := filepath.Join(modulePath, "my-lesson.md")
	if _, err := os.Stat(lessonPath); os.IsNotExist(err) {
		t.Errorf("lesson file not created at %s", lessonPath)
	}
}

func TestCommandBlog(t *testing.T) {
	tmpDir := t.TempDir()
	oldContentDir := contentDir
	contentDir = tmpDir
	defer func() { contentDir = oldContentDir }()

	oldTemplatesDir := templatesDir
	templatesDir = filepath.Join(tmpDir, "templates")
	_ = os.MkdirAll(templatesDir, 0755)
	defer func() { templatesDir = oldTemplatesDir }()

	_ = os.MkdirAll(filepath.Join(tmpDir, "blog"), 0755)

	// Inputs: title, summary
	defaultReader = bufio.NewReader(strings.NewReader("My Blog Post\nThis is a blog post\n"))

	createBlog("my-blog-post")

	blogPath := filepath.Join(tmpDir, "blog", "my-blog-post.md")
	if _, err := os.Stat(blogPath); os.IsNotExist(err) {
		t.Errorf("blog file not created at %s", blogPath)
	}
}

func TestCommandDoc(t *testing.T) {
	tmpDir := t.TempDir()
	oldContentDir := contentDir
	contentDir = tmpDir
	defer func() { contentDir = oldContentDir }()

	oldTemplatesDir := templatesDir
	templatesDir = filepath.Join(tmpDir, "templates")
	_ = os.MkdirAll(templatesDir, 0755)
	defer func() { templatesDir = oldTemplatesDir }()

	_ = os.MkdirAll(filepath.Join(tmpDir, "docs"), 0755)

	// Inputs: title, summary, weight
	defaultReader = bufio.NewReader(strings.NewReader("My Doc\nThis is a doc\n5\n"))

	createDoc("my-doc")

	docPath := filepath.Join(tmpDir, "docs", "my-doc.md")
	if _, err := os.Stat(docPath); os.IsNotExist(err) {
		t.Errorf("doc file not created at %s", docPath)
	}
}
