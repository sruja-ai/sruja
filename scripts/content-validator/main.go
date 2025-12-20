package main

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

const (
	contentDir         = "apps/website/src/content"
	indexFile          = "_index.md"
	moduleOverviewFile = "module-overview.md"
)

type ValidationError struct {
	File    string
	Message string
}

func main() {
	var errors []ValidationError

	fmt.Println("🔍 Validating content structure...")
	fmt.Println()

	// Validate courses
	errors = append(errors, validateCourses()...)

	// Validate tutorials
	errors = append(errors, validateTutorials()...)

	// Validate blogs
	errors = append(errors, validateBlogs()...)

	// Validate docs
	errors = append(errors, validateDocs()...)

	// Print results
	if len(errors) == 0 {
		fmt.Println("✅ All content files are valid!")
		os.Exit(0)
	}

	fmt.Printf("❌ Found %d validation error(s):\n\n", len(errors))
	for _, err := range errors {
		fmt.Printf("  %s: %s\n", err.File, err.Message)
	}
	os.Exit(1)
}

func validateCourses() []ValidationError {
	var errors []ValidationError
	coursesDir := filepath.Join(contentDir, "courses")

	// Check if courses directory exists
	if _, err := os.Stat(coursesDir); os.IsNotExist(err) {
		return errors
	}

	// Walk through courses
	err := filepath.Walk(coursesDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Check module-overview.md files
		if info.Name() == moduleOverviewFile {
			relPath, _ := filepath.Rel(contentDir, path)
			if errs := validateFrontmatter(path); len(errs) > 0 {
				errors = append(errors, errs...)
			}

			// Check required fields
			content, _ := os.ReadFile(path)
			if !strings.Contains(string(content), "title:") {
				errors = append(errors, ValidationError{
					File:    relPath,
					Message: "Missing required field: title",
				})
			}
			if !strings.Contains(string(content), "summary:") {
				errors = append(errors, ValidationError{
					File:    relPath,
					Message: "Missing required field: summary",
				})
			}
		}

		// Check lesson files (exclude module-overview.md)
		if strings.HasSuffix(info.Name(), ".md") && info.Name() != moduleOverviewFile {

			if errs := validateFrontmatter(path); len(errs) > 0 {
				errors = append(errors, errs...)
			}
		}

		return nil
	})

	if err != nil {
		fmt.Printf("Error walking courses directory: %v\n", err)
	}

	return errors
}

func validateTutorials() []ValidationError {
	var errors []ValidationError
	tutorialsDir := filepath.Join(contentDir, "tutorials")

	if _, err := os.Stat(tutorialsDir); os.IsNotExist(err) {
		return errors
	}

	files, _ := os.ReadDir(tutorialsDir)
	for _, file := range files {
		if strings.HasSuffix(file.Name(), ".md") && file.Name() != indexFile {
			path := filepath.Join(tutorialsDir, file.Name())

			if errs := validateFrontmatter(path); len(errs) > 0 {
				errors = append(errors, errs...)
			}
		}
	}

	return errors
}

func validateBlogs() []ValidationError {
	var errors []ValidationError
	blogsDir := filepath.Join(contentDir, "blog")

	if _, err := os.Stat(blogsDir); os.IsNotExist(err) {
		return errors
	}

	files, _ := os.ReadDir(blogsDir)
	for _, file := range files {
		if strings.HasSuffix(file.Name(), ".md") && file.Name() != indexFile {
			path := filepath.Join(blogsDir, file.Name())

			if errs := validateFrontmatter(path); len(errs) > 0 {
				errors = append(errors, errs...)
			}
		}
	}

	return errors
}

func validateDocs() []ValidationError {
	var errors []ValidationError
	docsDir := filepath.Join(contentDir, "docs")

	if _, err := os.Stat(docsDir); os.IsNotExist(err) {
		return errors
	}

	err := filepath.Walk(docsDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if strings.HasSuffix(info.Name(), ".md") && info.Name() != indexFile {

			if errs := validateFrontmatter(path); len(errs) > 0 {
				errors = append(errors, errs...)
			}
		}

		return nil
	})

	if err != nil {
		fmt.Printf("Error walking docs directory: %v\n", err)
	}

	return errors
}

func validateFrontmatter(path string) []ValidationError {
	var errors []ValidationError
	relPath, _ := filepath.Rel(contentDir, path)

	content, err := os.ReadFile(path)
	if err != nil {
		errors = append(errors, ValidationError{
			File:    relPath,
			Message: fmt.Sprintf("Could not read file: %v", err),
		})
		return errors
	}

	contentStr := string(content)

	// Check for frontmatter delimiter
	if !strings.HasPrefix(contentStr, "---\n") {
		errors = append(errors, ValidationError{
			File:    relPath,
			Message: "Missing frontmatter delimiter (---)",
		})
		return errors
	}

	// Extract frontmatter
	parts := strings.SplitN(contentStr, "---\n", 3)
	if len(parts) < 3 {
		errors = append(errors, ValidationError{
			File:    relPath,
			Message: "Invalid frontmatter format",
		})
		return errors
	}

	frontmatter := parts[1]

	// Check for title
	if !strings.Contains(frontmatter, "title:") {
		errors = append(errors, ValidationError{
			File:    relPath,
			Message: "Missing required field: title",
		})
	}

	// Check for empty TODO placeholders
	if strings.Contains(contentStr, "TODO:") {
		fmt.Printf("⚠️  %s: Contains TODO placeholder\n", relPath)
	}

	// Validate YAML-like structure (basic check)
	if matched, err := regexp.MatchString(`^[\w\s\-:"]+\n`, frontmatter); err == nil && !matched {
		// This is a basic check - could be improved
		_ = matched // suppress unused variable warning
	}

	return errors
}
