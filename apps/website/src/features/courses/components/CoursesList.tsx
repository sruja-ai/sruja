// apps/website/src/features/courses/components/CoursesList.tsx
import '@sruja/ui/design-system/styles.css';
import { EmptyState } from '@/shared/components/ui/EmptyState';
import { useExpansion } from '@/shared/hooks/useExpansion';

interface Lesson {
  slug: string;
  title: string;
  summary?: string;
  weight?: number;
}

interface Module {
  name: string;
  title: string;
  lessons: Lesson[];
}

interface Course {
  name: string;
  title: string;
  modules: Module[];
}

interface CoursesListProps {
  courses: Course[];
}

export default function CoursesList({ courses }: CoursesListProps) {
  const { isExpanded, toggle } = useExpansion<string>();

  if (courses.length === 0) {
    return (
      <EmptyState message="No courses available yet. Check back soon!" />
    );
  }

  return (
    <div className="courses-list">
      {courses.map((course) => {
        const courseExpanded = isExpanded(course.name);
        return (
          <article key={course.name} id={`course-${course.name}`} className="course-card">
            <div
              className="course-header"
              onClick={() => toggle(course.name)}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  toggle(course.name);
                }
              }}
            >
              <a
                href={`/courses/${course.name}`}
                className="course-title-link"
                onClick={(e) => {
                  e.stopPropagation();
                }}
              >
                <h2 className="course-title">{course.title}</h2>
              </a>
              <svg
                width="20"
                height="20"
                viewBox="0 0 20 20"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className={`expand-icon ${courseExpanded ? 'expanded' : ''}`}
              >
                <path d="m6 8 4 4 4-4" />
              </svg>
            </div>
            {courseExpanded && (
              <div className="course-modules">
                {course.modules.map((module) => (
                  <div key={module.name} className="module">
                    <h3 className="module-title">{module.title}</h3>
                    <ul className="lessons-list">
                      {module.lessons.map((lesson) => (
                        <li key={lesson.slug} className="lesson-item">
                          <a href={`/courses/${lesson.slug}`} className="lesson-link">
                            {lesson.title}
                          </a>
                          {lesson.summary && (
                            <span className="lesson-summary">{lesson.summary}</span>
                          )}
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            )}
          </article>
        );
      })}
    </div>
  );
}
