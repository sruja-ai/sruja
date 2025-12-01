// Course state management using localStorage
import type { CourseState } from '../types';
import { createSafeAnchor } from './sanitize';
import { getStorageJSON, setStorageJSON } from './storage';

const COURSE_KEY = 'sruja_course_state';

const DEFAULT_STATE: CourseState = {
  visited: [],
  quizResults: {},
  lastVisited: null
};

export function getCourseState(): CourseState {
  return getStorageJSON<CourseState>(COURSE_KEY, DEFAULT_STATE) || DEFAULT_STATE;
}

export function saveCourseState(state: CourseState): void {
  setStorageJSON(COURSE_KEY, state);
}

export function trackPageVisit(): void {
  const path = window.location.pathname;
  if (!path.includes('/courses/')) {
    return;
  }

  const state = getCourseState();
  if (!state.visited.includes(path)) {
    state.visited.push(path);
  }
  state.lastVisited = path;
  saveCourseState(state);
  updateProgressUI();
}

function updateProgressUI(): void {
  const state = getCourseState();
  const links = document.querySelectorAll('a');
  links.forEach(link => {
    const href = link.getAttribute('href');
    if (href && state.visited.some(v => href.endsWith(v) || v.endsWith(href))) {
      link.classList.add('visited');
    }
  });

  const resumeContainer = document.getElementById('resume-course-container');
  if (resumeContainer && state.lastVisited) {
    resumeContainer.textContent = ''; // Clear existing content
    const link = createSafeAnchor(state.lastVisited, 'Resume Course', 'btn btn-primary');
    resumeContainer.appendChild(link);
  }
}

export function submitQuiz(quizId: string, answers: unknown): void {
  const state = getCourseState();
  state.quizResults[quizId] = answers;
  saveCourseState(state);
}

