// Course State Management using localStorage

const COURSE_KEY = 'sruja_course_state';

// Inject Custom CSS
const link = document.createElement('link');
link.rel = 'stylesheet';
link.href = '/css/course.css';
document.head.appendChild(link);

// Initialize state if not present
function getCourseState() {
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

function saveCourseState(state) {
    localStorage.setItem(COURSE_KEY, JSON.stringify(state));
}

// Track page visit
function trackPageVisit() {
    const path = window.location.pathname;
    // Only track course pages
    if (!path.includes('/course/')) {
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

// Update UI with progress
function updateProgressUI() {
    const state = getCourseState();
    // Example: Add a checkmark to sidebar links
    const links = document.querySelectorAll('a');
    links.forEach(link => {
        const href = link.getAttribute('href');
        if (href && state.visited.some(v => href.endsWith(v) || v.endsWith(href))) {
            link.classList.add('visited');
            // Optional: Add checkmark icon
            // link.innerHTML += ' ✓'; 
        }
    });

    // Show "Resume Course" button on course home
    const resumeContainer = document.getElementById('resume-course-container');
    if (resumeContainer && state.lastVisited) {
        resumeContainer.innerHTML = `<a href="${state.lastVisited}" class="btn btn-primary">Resume Course</a>`;
    }
}

// Quiz Logic
function submitQuiz(quizId, answers) {
    const state = getCourseState();
    state.quizResults[quizId] = answers;
    saveCourseState(state);
}

document.addEventListener('DOMContentLoaded', () => {
    trackPageVisit();
});
