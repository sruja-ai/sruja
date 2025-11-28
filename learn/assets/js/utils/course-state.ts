// Course state management using localStorage
import type { CourseState } from '../types';

const COURSE_KEY = 'sruja_course_state';

export function getCourseState(): CourseState {
  const state = localStorage.getItem(COURSE_KEY);
  if (state) {
    return JSON.parse(state);
  }
  return {
    visited: [],
    quizResults: {},
    lastVisited: null
  };
}

export function saveCourseState(state: CourseState): void {
  localStorage.setItem(COURSE_KEY, JSON.stringify(state));
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
    resumeContainer.innerHTML = `<a href="${state.lastVisited}" class="btn btn-primary">Resume Course</a>`;
  }
}

export function submitQuiz(quizId: string, answers: unknown): void {
  const state = getCourseState();
  state.quizResults[quizId] = answers;
  saveCourseState(state);
}

