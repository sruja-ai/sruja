/**
 * Progress Tracker for Sruja Courses
 * Tracks lesson visits and quiz completions, stores in localStorage,
 * and updates the UI with progress bar and completion indicators.
 */

(function() {
  'use strict';

  // Configuration
  const STORAGE_KEY = 'sruja-course-progress';
  const PROGRESS_SELECTOR = '#sruja-progress-container';
  const SIDEBAR_SELECTOR = '.sidebar .chapter-item';
  const SIDEBAR_ACTIVE_SELECTOR = '.sidebar .chapter-item.active > .chapter-item-flex';
  const SIDEBAR_PARENT_SELECTOR = '.sidebar .chapter-item';

  // State
  let progressData = {
    courses: {}
  };

  // Initialize
  function init() {
    loadProgress();
    createProgressBar();
    trackCurrentLesson();
    updateSidebarIndicators();
    observeQuizCompletion();
    observeNavigation();
    setupResetButton();
  }

  // Load progress from localStorage
  function loadProgress() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        progressData = JSON.parse(saved);
      }
    } catch (e) {
      console.error('Failed to load progress:', e);
      progressData = { courses: {} };
    }
  }

  // Save progress to localStorage
  function saveProgress() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(progressData));
    } catch (e) {
      console.error('Failed to save progress:', e);
    }
  }

  // Create progress bar in the UI
  function createProgressBar() {
    // Only show progress bar on course pages, not on documentation
    const isCoursePage = window.location.pathname.includes('/courses/');
    if (!isCoursePage) return;

    // Check if already exists
    if (document.querySelector(PROGRESS_SELECTOR)) return;

    // Find the sidebar or main content area
    const sidebar = document.querySelector('.sidebar');
    if (!sidebar) return;

    // Create progress container
    const progressContainer = document.createElement('div');
    progressContainer.id = 'sruja-progress-container';
    progressContainer.innerHTML = `
      <div class="sruja-progress-header">
        <span class="sruja-progress-title">Course Progress</span>
        <button id="sruja-reset-progress" class="sruja-reset-btn" title="Reset Progress">↺</button>
      </div>
      <div class="sruja-progress-bar-container">
        <div id="sruja-progress-bar" class="sruja-progress-bar" style="width: 0%"></div>
      </div>
      <div id="sruja-progress-text" class="sruja-progress-text">0% complete</div>
    `;

    // Insert at the top of sidebar
    const sidebarContent = sidebar.querySelector('.sidebar-scrollbox') || sidebar;
    sidebarContent.insertBefore(progressContainer, sidebarContent.firstChild);

    updateProgressBar();
  }

  // Update progress bar with current progress
  function updateProgressBar() {
    const progressBar = document.querySelector('#sruja-progress-bar');
    const progressText = document.querySelector('#sruja-progress-text');

    if (!progressBar || !progressText) return;

    const courseProgress = getCourseProgress();
    const percentage = Math.round(courseProgress.percentage);

    progressBar.style.width = percentage + '%';
    progressText.textContent = `${percentage}% complete (${courseProgress.completed}/${progressBar.dataset.total || '?'} lessons)`;

    // Add color classes based on progress
    progressBar.classList.remove('sruja-progress-low', 'sruja-progress-medium', 'sruja-progress-high');
    if (percentage < 25) {
      progressBar.classList.add('sruja-progress-low');
    } else if (percentage < 75) {
      progressBar.classList.add('sruja-progress-medium');
    } else {
      progressBar.classList.add('sruja-progress-high');
    }
  }

  // Get current course progress
  function getCourseProgress() {
    const courseName = getCourseName();
    if (!courseName) return { completed: 0, total: 0, percentage: 0 };

    const courseProgress = progressData.courses[courseName] || {};
    const completedLessons = Object.keys(courseProgress).filter(key => courseProgress[key].visited);
    const completedQuizzes = Object.keys(courseProgress).filter(key => courseProgress[key].quizCompleted);

    // Count total lessons in the current course
    const totalLessons = getTotalLessonsInCourse();

    return {
      completed: completedLessons.length,
      completedWithQuizzes: completedQuizzes.length,
      total: totalLessons,
      percentage: totalLessons > 0 ? (completedLessons.length / totalLessons) * 100 : 0
    };
  }

  // Get course name from URL
  function getCourseName() {
    const pathParts = window.location.pathname.split('/');
    const coursesIndex = pathParts.indexOf('courses');
    if (coursesIndex === -1 || coursesIndex + 1 >= pathParts.length) return null;
    return pathParts[coursesIndex + 1];
  }

  // Get module name from URL
  function getModuleName() {
    const pathParts = window.location.pathname.split('/');
    const coursesIndex = pathParts.indexOf('courses');
    if (coursesIndex === -1 || coursesIndex + 2 >= pathParts.length) return null;
    return pathParts[coursesIndex + 2];
  }

  // Get lesson name from URL
  function getLessonName() {
    const path = window.location.pathname;
    const match = path.match(/lesson-\d+\.md/);
    return match ? match[0] : null;
  }

  // Count total lessons in the current course by checking sidebar
  function getTotalLessonsInCourse() {
    const lessonLinks = document.querySelectorAll('.sidebar a[href*="lesson-"]');
    return lessonLinks.length;
  }

  // Track the current lesson as visited
  function trackCurrentLesson() {
    const courseName = getCourseName();
    const moduleName = getModuleName();
    const lessonName = getLessonName();

    if (!courseName || !moduleName || !lessonName) return;

    const lessonId = `${courseName}/${moduleName}/${lessonName}`;

    // Initialize course data if needed
    if (!progressData.courses[courseName]) {
      progressData.courses[courseName] = {};
    }

    // Mark lesson as visited
    if (!progressData.courses[courseName][lessonId]) {
      progressData.courses[courseName][lessonId] = {
        visited: false,
        quizCompleted: false,
        timestamp: null
      };
    }

    // Only update if not already visited (don't update timestamp on every revisit)
    if (!progressData.courses[courseName][lessonId].visited) {
      progressData.courses[courseName][lessonId].visited = true;
      progressData.courses[courseName][lessonId].timestamp = Date.now();
      saveProgress();
      updateProgressBar();
      updateSidebarIndicators();
    }
  }

  // Add completion indicators to sidebar
  function updateSidebarIndicators() {
    const courseName = getCourseName();
    if (!courseName) return;

    const courseProgress = progressData.courses[courseName] || {};
    const lessonLinks = document.querySelectorAll('.sidebar a[href*="lesson-"]');

    lessonLinks.forEach(link => {
      const href = link.getAttribute('href');
      const match = href.match(/courses\/([^/]+)\/([^/]+)\/([^/]+)/);
      if (!match) return;

      const linkCourse = match[1];
      const linkModule = match[2];
      const linkLesson = match[3];
      const lessonId = `${linkCourse}/${linkModule}/${linkLesson}`;

      // Only process lessons from the current course
      if (linkCourse !== courseName) return;

      const progress = courseProgress[lessonId];
      const parentChapterItem = link.closest('.chapter-item');

      // Remove existing indicators
      const existingIndicator = parentChapterItem.querySelector('.sruja-lesson-indicator');
      if (existingIndicator) {
        existingIndicator.remove();
      }

      // Add indicator based on progress
      if (progress) {
        const indicator = document.createElement('span');
        indicator.className = 'sruja-lesson-indicator';

        if (progress.quizCompleted) {
          indicator.innerHTML = '✓';
          indicator.classList.add('sruja-lesson-complete');
          indicator.title = 'Lesson and quiz completed';
        } else if (progress.visited) {
          indicator.innerHTML = '○';
          indicator.classList.add('sruja-lesson-visited');
          indicator.title = 'Lesson read, quiz not completed';
        }

        // Add indicator after the link text
        const flexContainer = parentChapterItem.querySelector('.chapter-item-flex');
        if (flexContainer) {
          flexContainer.appendChild(indicator);
        } else {
          link.parentNode.appendChild(indicator);
        }
      }
    });
  }

  // Observe quiz completion events (mdbook-quiz)
  function observeQuizCompletion() {
    // Listen for custom events from mdbook-quiz
    document.addEventListener('quiz-completed', handleQuizCompleted);

    // Also poll for quiz completion (fallback)
    checkQuizCompletion();
  }

  // Handle quiz completion event
  function handleQuizCompleted(event) {
    const courseName = getCourseName();
    const moduleName = getModuleName();
    const lessonName = getLessonName();

    if (!courseName || !moduleName || !lessonName) return;

    const lessonId = `${courseName}/${moduleName}/${lessonName}`;

    if (!progressData.courses[courseName]) {
      progressData.courses[courseName] = {};
    }

    if (!progressData.courses[courseName][lessonId]) {
      progressData.courses[courseName][lessonId] = {
        visited: false,
        quizCompleted: false,
        timestamp: null
      };
    }

    progressData.courses[courseName][lessonId].quizCompleted = true;
    saveProgress();
    updateProgressBar();
    updateSidebarIndicators();
  }

  // Check for quiz completion by polling (fallback)
  function checkQuizCompletion() {
    // Check periodically if quiz is completed
    setInterval(() => {
      // Look for quiz elements that indicate completion
      const quizElements = document.querySelectorAll('.quiz-container');

      quizElements.forEach(quiz => {
        // If all questions are answered correctly, mark as completed
        const allCorrect = quiz.querySelectorAll('.quiz-question.correct');
        const totalQuestions = quiz.querySelectorAll('.quiz-question');

        if (allCorrect.length > 0 && allCorrect.length === totalQuestions.length) {
          handleQuizCompleted({ detail: { score: 100 } });
        }
      });
    }, 2000); // Check every 2 seconds
  }

  // Observe navigation changes (for single-page app behavior)
  function observeNavigation() {
    // Listen for popstate (back/forward buttons)
    window.addEventListener('popstate', () => {
      setTimeout(() => {
        createProgressBar();
        trackCurrentLesson();
        updateSidebarIndicators();
      }, 100);
    });

    // Listen for hash changes
    window.addEventListener('hashchange', () => {
      // Hash changes don't trigger page reload, but we're interested in full navigation
    });
  }

  // Setup reset button
  function setupResetButton() {
    const resetBtn = document.getElementById('sruja-reset-progress');
    if (!resetBtn) return;

    resetBtn.addEventListener('click', function(e) {
      e.preventDefault();

      const courseName = getCourseName();
      if (!courseName) return;

      if (confirm('Are you sure you want to reset all progress for this course?')) {
        progressData.courses[courseName] = {};
        saveProgress();
        updateProgressBar();
        updateSidebarIndicators();

        // Visual feedback
        resetBtn.textContent = '✓';
        setTimeout(() => {
          resetBtn.textContent = '↺';
        }, 1000);
      }
    });
  }

  // Initialize on DOM ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

})();
